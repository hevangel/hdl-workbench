import {
  ArtifactIdSchema,
  CompileIdSchema,
  ConnectionStatusSchema,
  DiagnosticSchema,
  ExecuteRequestSchema,
  ProjectPathSchema,
  RunIdSchema,
  RunResultSchema,
  TimescaleSchema,
  type ArtifactId,
  type CompileId,
  type CompileRequest,
  type CompileResult,
  type ConnectionStatus,
  type Diagnostic,
  type ExecuteRequest,
  type FileEntry,
  type IdeController,
  type ProjectId,
  type ProjectPath,
  type Revision,
  type RunId,
  type RunRequest,
  type RunResult,
  type SurferAdapter,
  type WaveformArtifact,
  type WaveformTime,
  type WorkspaceAdapter,
} from "@xezim-ide/protocol";
import { runXezimProjectWorker } from "../workers/xezimClient.js";
import type { ProjectFileInput } from "../workers/xezimProtocol.js";
import { OpfsWaveformStore } from "./OpfsWaveformStore.js";

export class BrowserControllerError extends Error {
  constructor(readonly code: string, message: string, readonly retryable = false) {
    super(message);
    this.name = "BrowserControllerError";
  }
}

export interface BrowserIdeControllerOptions {
  simulatorAvailable?: boolean;
  requestTimeoutMs?: number;
  artifactStore?: OpfsWaveformStore;
  runProjectWorker?: typeof runXezimProjectWorker;
  onWaveformShown?: () => void;
}

const unitNanoseconds: Record<string, bigint> = {
  s: 1_000_000_000n,
  ms: 1_000_000n,
  us: 1_000n,
  ns: 1n,
  ps: 1n,
  fs: 1n,
};
const subNanosecondDivisor: Record<string, bigint> = { ps: 1_000n, fs: 1_000_000n };

function maxTimeInNanoseconds(until: WaveformTime, timescale: { magnitude: string; unit: string }): number {
  const ticks = BigInt(until.ticks);
  const magnitude = BigInt(timescale.magnitude);
  const factor = unitNanoseconds[timescale.unit];
  if (factor === undefined) throw new BrowserControllerError("INVALID_REQUEST", `Unsupported time unit ${timescale.unit}.`);
  let nanoseconds = ticks * magnitude * factor;
  const divisor = subNanosecondDivisor[timescale.unit];
  if (divisor) {
    const subNanoseconds = ticks * magnitude;
    if (subNanoseconds % divisor !== 0n) {
      throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", "This Xezim Worker accepts integer nanosecond stop times; the requested time is fractional nanoseconds.");
    }
    nanoseconds = subNanoseconds / divisor;
  }
  if (nanoseconds > 10_000_000n) {
    throw new BrowserControllerError("RESOURCE_LIMIT", "This Xezim Worker supports stop times up to 10,000,000 ns.");
  }
  return Number(nanoseconds);
}

function parseTimescale(raw: string | null | undefined): { magnitude: string; unit: "s" | "ms" | "us" | "ns" | "ps" | "fs" } {
  const match = /^(\d{1,3})\s*(s|ms|us|ns|ps|fs)$/.exec(raw?.trim() ?? "");
  const parsed = match && TimescaleSchema.safeParse({ magnitude: match[1], unit: match[2] });
  if (!parsed?.success) throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", `Xezim did not return a supported VCD timescale (received ${JSON.stringify(raw ?? null)}); the run result cannot be labeled safely.`);
  return parsed.data;
}

function bridgeSnapshotId(files: Array<{ path: string; revision: string }>): Promise<string> {
  const source = files.map((file) => `${file.path}\0${file.revision}\0`).join("");
  return crypto.subtle.digest("SHA-256", new TextEncoder().encode(source)).then((hash) => {
    const hex = [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    return `snap-${hex.slice(0, 32)}`;
  });
}

function toWorkerFiles(files: Array<{ path: string; bytes: Uint8Array }>): ProjectFileInput[] {
  const decoder = new TextDecoder("utf-8", { fatal: true });
  try {
    return files.map(({ path, bytes }) => ({ path, contents: decoder.decode(bytes) }));
  } catch {
    throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", "The current browser Xezim API accepts UTF-8 text project files only.");
  }
}

function runId(): RunId {
  return RunIdSchema.parse(`run-${crypto.randomUUID()}`);
}

function artifactId(): ArtifactId {
  return ArtifactIdSchema.parse(`artifact-${crypto.randomUUID()}`);
}

/** Browser-side controller shared by the Run button and authenticated RPC calls. */
export class BrowserIdeController implements IdeController {
  private surfer: SurferAdapter | null = null;
  private readonly recentRuns = new Map<RunId, RunResult>();
  private readonly diagnosticsByRun = new Map<RunId, Diagnostic[]>();
  private readonly options: BrowserIdeControllerOptions;
  private simulationBusy = false;
  private waveformShownListener: (() => void) | undefined;
  private workspaceChangedListener: (() => void) | undefined;
  readonly artifacts: OpfsWaveformStore;

  constructor(public workspace: WorkspaceAdapter, options: BrowserIdeControllerOptions = {}) {
    this.options = options;
    this.artifacts = options.artifactStore ?? new OpfsWaveformStore();
    this.waveformShownListener = options.onWaveformShown;
  }

  setSurferAdapter(adapter: SurferAdapter | null): void {
    this.surfer = adapter;
  }

  setWorkspace(workspace: WorkspaceAdapter): void {
    this.workspace = workspace;
  }

  setOnWaveformShown(callback: (() => void) | undefined): void {
    this.waveformShownListener = callback;
  }

  setOnWorkspaceChanged(callback: (() => void) | undefined): void {
    this.workspaceChangedListener = callback;
  }

  get canExecute(): boolean { return this.options.simulatorAvailable !== false; }

  connectionStatus(connected = false): Promise<ConnectionStatus> {
    const simulatorUnavailable = this.options.simulatorAvailable === false;
    return Promise.resolve(ConnectionStatusSchema.parse({
      protocol: "xezim-ide/1",
      connected,
      capabilities: [
        {
          name: "browser_simulation",
          available: !simulatorUnavailable,
          reason: simulatorUnavailable
            ? "The generated Xezim WASM assets are unavailable. Build the pinned xezim-web target; no native simulator fallback is configured."
            : "Xezim uses its verified one-shot compile-and-run Worker API; separate compile and run handles are unavailable.",
        },
        { name: "waveform_load", available: this.surfer !== null, ...(this.surfer ? {} : { reason: "Open the Waveforms panel to initialize the pinned Surfer viewer." }) },
        { name: "waveform_add_signals", available: this.surfer !== null, ...(this.surfer ? {} : { reason: "Open the Waveforms panel to initialize the pinned Surfer viewer." }) },
        { name: "waveform_cursor", available: this.surfer !== null, ...(this.surfer ? {} : { reason: "Open the Waveforms panel to initialize the pinned Surfer viewer." }) },
      ],
    }));
  }

  listFiles(): Promise<FileEntry[]> { return this.workspace.listFiles(); }
  readFile(path: ProjectPath): Promise<{ bytes: Uint8Array; revision: Revision }> { return this.workspace.readFile(path); }
  async writeFile(path: ProjectPath, bytes: Uint8Array, expectedRevision: Revision | null): Promise<{ revision: Revision }> {
    const result = await this.workspace.writeFile(path, bytes, expectedRevision);
    this.workspaceChangedListener?.();
    return result;
  }
  async deleteFile(path: ProjectPath, expectedRevision: Revision): Promise<void> {
    await this.workspace.deleteFile(path, expectedRevision);
    this.workspaceChangedListener?.();
  }

  async compile(_input: CompileRequest): Promise<CompileResult> {
    throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", "The pinned browser Xezim API does not expose compile-only operations. Use the one-shot Run command, which compiles and simulates in the same Worker call.");
  }

  async run(_input: RunRequest): Promise<RunResult> {
    throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", "The pinned browser Xezim API does not expose a compiled design handle. Use the one-shot Run command, which compiles and simulates in the same Worker call.");
  }

  async execute(input: ExecuteRequest): Promise<RunResult> {
    if (this.simulationBusy) throw new BrowserControllerError("SIMULATION_BUSY", "An Xezim Worker run is already active.", true);
    this.simulationBusy = true;
    try {
      return await this.executeNow(input);
    } finally {
      this.simulationBusy = false;
    }
  }

  private async executeNow(input: ExecuteRequest): Promise<RunResult> {
    const request = ExecuteRequestSchema.parse(input);
    if (this.options.simulatorAvailable === false) {
      throw new BrowserControllerError("SIMULATOR_UNAVAILABLE", "Xezim WASM assets are not available in this browser build. No native simulator fallback is configured.");
    }
    const snapshot = await this.workspace.snapshot();
    const serverSnapshot = await bridgeSnapshotId(snapshot.files);
    if (request.snapshotId !== snapshot.snapshotId && request.snapshotId !== serverSnapshot) {
      throw new BrowserControllerError("REVISION_CONFLICT", "The browser workspace no longer matches the requested project snapshot. Synchronize and retry.", true);
    }
    const timeNs = maxTimeInNanoseconds(request.until, request.until.timescale);
    const sources = toWorkerFiles(snapshot.files);
    const currentRunId = runId();
    let response;
    try {
      response = await (this.options.runProjectWorker ?? runXezimProjectWorker)({
        files: sources,
        snapshotId: request.snapshotId,
        topModule: request.topModule,
        maxTimeNs: timeNs,
        maxOutputBytes: request.workBudget.maxOutputBytes,
        maxVcdBytes: request.workBudget.maxVcdBytes,
      }, { timeoutMs: this.options.requestTimeoutMs ?? request.workBudget.maxWallTimeMs });
    } catch (error) {
      throw new BrowserControllerError("INTERNAL_ERROR", error instanceof Error ? error.message : "The Xezim Worker request failed.");
    }
    if (!response.ok) {
      const code = response.error.code === "TIMEOUT" ? "TIMEOUT"
        : response.error.code === "RESOURCE_LIMIT" ? "RESOURCE_LIMIT"
          : "SIMULATOR_UNAVAILABLE";
      throw new BrowserControllerError(code, response.error.message, code === "TIMEOUT");
    }
    const raw = response.result;
    if (raw.snapshotId !== request.snapshotId) throw new BrowserControllerError("INTERNAL_ERROR", "Xezim returned a result for a different source snapshot.");
    const timescale = parseTimescale(raw.waveform?.timescale);
    const diagnostics = raw.diagnostics.map((item) => DiagnosticSchema.parse({
      severity: item.severity,
      code: item.code,
      message: item.message,
    }));
    let artifact: WaveformArtifact | undefined;
    if (raw.vcd && raw.waveform?.complete && raw.waveform.sizeBytes > 0) {
      const nextArtifact = {
        id: artifactId(),
        runId: currentRunId,
        snapshotId: request.snapshotId,
        format: "vcd" as const,
        sizeBytes: new TextEncoder().encode(raw.vcd).byteLength,
        storage: "opfs" as const,
        timescale,
        complete: true,
      };
      await this.artifacts.put(nextArtifact, raw.vcd);
      artifact = nextArtifact;
    }
    await this.applyFileOutputs(raw.fileOutputs ?? [], snapshot.files);
    const result = RunResultSchema.parse({
      runId: currentRunId,
      projectId: request.projectId,
      snapshotId: request.snapshotId,
      status: raw.status === "resource_limit" ? "resource_limit"
        : raw.status === "failed" ? "failed"
          : raw.finished ? "finish" : "completed",
      diagnostics,
      stdout: raw.stdout,
      stderr: "",
      lastTime: { ticks: raw.lastTimeTicks, timescale },
      ...(artifact ? { waveform: artifact } : {}),
    });
    this.recentRuns.set(result.runId, result);
    this.diagnosticsByRun.set(result.runId, result.diagnostics);
    while (this.recentRuns.size > 100) {
      const oldest = this.recentRuns.keys().next().value as RunId | undefined;
      if (!oldest) break;
      this.recentRuns.delete(oldest);
      this.diagnosticsByRun.delete(oldest);
    }
    return result;
  }

  async diagnostics(input: { compileId?: CompileId; runId?: RunId }): Promise<Diagnostic[]> {
    if (input.compileId) throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", "Compile-only diagnostics are unavailable because Xezim uses a combined compile-and-run operation.");
    if (input.runId) return this.diagnosticsByRun.get(input.runId) ?? [];
    return [...this.diagnosticsByRun.values()].flat();
  }

  async listRuns(projectId: ProjectId, limit: number): Promise<RunResult[]> {
    return [...this.recentRuns.values()].filter((run) => run.projectId === projectId).slice(-Math.max(1, limit)).reverse();
  }

  async showWaveform(input: { artifactId: ArtifactId; signalPaths?: string[]; cursor?: WaveformTime }): Promise<void> {
    if (!this.surfer) throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", "The Surfer viewer is not initialized. Open the Waveforms panel and retry.");
    const artifact = await this.artifacts.get(input.artifactId);
    await this.surfer.loadWaveform(artifact);
    if (input.signalPaths?.length) await this.surfer.addSignals(input.signalPaths);
    if (input.cursor) await this.surfer.setCursor(input.cursor);
    this.waveformShownListener?.();
  }

  async getLog(input: { runId: RunId; tailBytes: number }): Promise<{ stdout: string; stderr: string }> {
    const run = this.recentRuns.get(input.runId);
    if (!run) throw new BrowserControllerError("NOT_FOUND", `Run ${input.runId} is not available in this browser session.`);
    const tail = (value: string) => {
      const bytes = new TextEncoder().encode(value);
      return new TextDecoder().decode(bytes.slice(Math.max(0, bytes.byteLength - input.tailBytes)));
    };
    return { stdout: tail(run.stdout), stderr: tail(run.stderr) };
  }

  dispose(): void {
    this.artifacts.dispose();
    this.surfer?.dispose();
    this.surfer = null;
  }

  private async applyFileOutputs(outputs: Array<{ path: string; contents: string }>, files: Array<{ path: ProjectPath; revision: Revision }>): Promise<void> {
    const revisions = new Map(files.map((file) => [file.path, file.revision]));
    for (const output of outputs) {
      const path = ProjectPathSchema.parse(output.path);
      const currentRevision = revisions.get(path) ?? null;
      await this.workspace.writeFile(path, new TextEncoder().encode(output.contents), currentRevision);
    }
    if (outputs.length > 0) this.workspaceChangedListener?.();
  }
}
