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
  compareProjectPaths,
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
import { compileDpiProjectInBrowser } from "../workers/dpiCompilerClient.js";
import type { DpiCompilerFile } from "../workers/dpiCompilerProtocol.js";
import type { DpiWasmModuleInput, ProjectFileInput } from "../workers/xezimProtocol.js";
import { OpfsWaveformStore } from "./OpfsWaveformStore.js";

const MAX_PROJECT_INPUT_BYTES = 16 * 1024 * 1024;
const MAX_PROJECT_FILES = 4096;

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
  compileDpiProjectWorker?: typeof compileDpiProjectInBrowser;
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
  const source = [...files]
    .sort((left, right) => compareProjectPaths(left.path, right.path))
    .map((file) => `${file.path}\0${file.revision}\0`)
    .join("");
  return crypto.subtle.digest("SHA-256", new TextEncoder().encode(source)).then((hash) => {
    const hex = [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    return `snap-${hex.slice(0, 32)}`;
  });
}

function toWorkerProject(files: Array<{ path: string; bytes: Uint8Array }>): {
  files: ProjectFileInput[];
  dpiModules: DpiWasmModuleInput[];
  dpiCompilerFiles: DpiCompilerFile[];
} {
  if (files.length === 0 || files.length > MAX_PROJECT_FILES) {
    throw new BrowserControllerError("RESOURCE_LIMIT", `Xezim accepts between 1 and ${MAX_PROJECT_FILES} project files.`);
  }
  const encoder = new TextEncoder();
  const totalBytes = files.reduce((sum, file) => sum + encoder.encode(file.path).byteLength + file.bytes.byteLength, 0);
  if (totalBytes > MAX_PROJECT_INPUT_BYTES) {
    throw new BrowserControllerError("RESOURCE_LIMIT", `Xezim project input exceeds ${MAX_PROJECT_INPUT_BYTES} bytes.`);
  }
  const decoder = new TextDecoder("utf-8", { fatal: true });
  try {
    const textFiles: ProjectFileInput[] = [];
    const dpiModules: DpiWasmModuleInput[] = [];
    const dpiCompilerFiles: DpiCompilerFile[] = [];
    for (const { path, bytes } of files) {
      const extension = path.split("/").at(-1)?.split(".").at(-1)?.toLowerCase();
      if (extension === "wasm") {
        const copy = new ArrayBuffer(bytes.byteLength);
        new Uint8Array(copy).set(bytes);
        dpiModules.push({ path, bytes: copy });
      } else {
        const contents = decoder.decode(bytes);
        textFiles.push({ path, contents });
        if (["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx", "inc"].includes(extension ?? "")) {
          dpiCompilerFiles.push({ path, contents });
        }
      }
    }
    return { files: textFiles, dpiModules, dpiCompilerFiles };
  } catch (error) {
    if (error instanceof BrowserControllerError) throw error;
    throw new BrowserControllerError("UNSUPPORTED_CAPABILITY", "The current browser Xezim API accepts UTF-8 text project files only.");
  }
}

function decodeBase64(encoded: string): Uint8Array {
  const binary = atob(encoded);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
  return bytes;
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
        {
          name: "browser_dpi_scalar_int32",
          available: !simulatorUnavailable,
          reason: simulatorUnavailable
            ? "The Xezim WASM assets are unavailable."
            : "The browser profile accepts standalone import-free Wasm modules and DPI functions with an int result and input int arguments.",
        },
        {
          name: "browser_c_cpp_compile",
          available: !simulatorUnavailable,
          reason: simulatorUnavailable
            ? "The Xezim WASM assets are unavailable."
            : "Pinned Clang/LLD WebAssembly compiler runs in a separate Worker. C/C++ modules must be self-contained, import-free, and use scalar int DPI signatures.",
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
    const workerProject = toWorkerProject(snapshot.files);
    const currentRunId = runId();
    const wallDeadline = Date.now() + request.workBudget.maxWallTimeMs;
    const remainingWallTime = () => Math.max(1, Math.min(
      this.options.requestTimeoutMs ?? request.workBudget.maxWallTimeMs,
      wallDeadline - Date.now(),
    ));
    const dpiModules = [...workerProject.dpiModules];
    if (workerProject.dpiCompilerFiles.some(({ path }) => ["c", "cc", "cpp", "cxx"].includes(path.split("/").at(-1)?.split(".").at(-1)?.toLowerCase() ?? ""))) {
      const compiled = await (this.options.compileDpiProjectWorker ?? compileDpiProjectInBrowser)(
        workerProject.dpiCompilerFiles,
        { timeoutMs: remainingWallTime() },
      );
      if (!compiled.ok) {
        if (compiled.error.code === "TIMEOUT") {
          throw new BrowserControllerError("TIMEOUT", compiled.error.message, true);
        }
        if (compiled.error.code === "RESOURCE_LIMIT") {
          throw new BrowserControllerError("RESOURCE_LIMIT", compiled.error.message);
        }
        if (compiled.error.code === "INVALID_PATH" || compiled.error.code === "INVALID_REQUEST") {
          throw new BrowserControllerError(compiled.error.code, compiled.error.message);
        }
        throw new BrowserControllerError("DPI_COMPILATION_FAILED", compiled.error.message);
      }
      dpiModules.push(...compiled.modules.map(({ path, bytes }) => ({ path: `${path}.compiled.wasm`, bytes })));
    }
    let response;
    try {
      response = await (this.options.runProjectWorker ?? runXezimProjectWorker)({
        files: workerProject.files,
        ...(dpiModules.length ? { dpiModules } : {}),
        snapshotId: request.snapshotId,
        topModule: request.topModule,
        maxTimeNs: timeNs,
        maxOutputBytes: request.workBudget.maxOutputBytes,
        maxVcdBytes: request.workBudget.maxVcdBytes,
        ...(request.plusargs?.length ? { plusargs: request.plusargs } : {}),
        ...(request.waveformFormat ? { waveformFormat: request.waveformFormat } : {}),
      }, { timeoutMs: remainingWallTime() });
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
    // Only designs that call $dumpfile/$dumpvars produce a VCD, and only a VCD carries a
    // timescale. A run without a dump is still a valid run, so label it with the request's.
    const timescale = raw.waveform ? parseTimescale(raw.waveform.timescale) : request.until.timescale;
    const diagnostics = raw.diagnostics.map((item) => DiagnosticSchema.parse({
      severity: item.severity,
      code: item.code,
      message: item.message,
    }));
    let artifact: WaveformArtifact | undefined;
    const fst = raw.fstBase64 ? decodeBase64(raw.fstBase64) : undefined;
    const dumped = fst ?? raw.vcd;
    if (dumped && raw.waveform?.complete && raw.waveform.sizeBytes > 0) {
      const nextArtifact = {
        id: artifactId(),
        runId: currentRunId,
        snapshotId: request.snapshotId,
        format: fst ? "fst" as const : "vcd" as const,
        sizeBytes: typeof dumped === "string" ? new TextEncoder().encode(dumped).byteLength : dumped.byteLength,
        storage: "opfs" as const,
        timescale,
        complete: true,
      };
      await this.artifacts.put(nextArtifact, dumped);
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
