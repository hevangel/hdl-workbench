import {
  CompileRequestSchema,
  CompileResultSchema,
  DiagnosticSchema,
  ExecuteRequestSchema,
  PairingChallengeSchema,
  ProjectPathSchema,
  RpcResultSchemas,
  RunResultSchema,
  type CompileId,
  type CompileResult,
  type Diagnostic,
  type ProjectId,
  type RunId,
  type RunResult,
} from "@xezim-ide/protocol";
import { BridgeError } from "./errors.js";
import { ProjectFiles } from "./projectFiles.js";
import { makeRpcRequest, WebSocketBridge } from "./websocketBridge.js";

export interface ServiceOptions {
  projectId?: ProjectId;
}

export class IdeMcpService {
  readonly projectId: ProjectId;
  private readonly compileResults = new Map<CompileId, CompileResult>();
  private readonly runResults = new Map<RunId, RunResult>();
  private readonly runOrder: RunId[] = [];
  private simulationBusy = false;

  constructor(readonly files: ProjectFiles, readonly bridge: WebSocketBridge, options: ServiceOptions = {}) {
    this.projectId = options.projectId ?? "local-project";
  }

  async getConnectionStatus(): Promise<Record<string, unknown>> {
    const pairing = this.bridge.status();
    let browserStatus: unknown;
    if (pairing.connected) {
      try {
        browserStatus = await this.success(makeRpcRequest({ op: "connection.get_status", projectId: this.projectId, params: {} }));
      } catch (error) {
        if (!(error instanceof BridgeError && error.code === "BROWSER_DISCONNECTED")) throw error;
      }
    }
    return {
      connected: pairing.connected && browserStatus !== undefined,
      websocketUrl: pairing.url,
      allowedOrigins: pairing.allowedOrigins,
      pairingChallenge: PairingChallengeSchema.parse(pairing.pairingChallenge),
      ...(browserStatus === undefined ? {} : { browserStatus }),
    };
  }

  async listProjectFiles(): Promise<unknown> {
    return { files: await this.files.list() };
  }

  async readProjectFile(path: string): Promise<unknown> {
    const file = await this.files.read(path);
    return { path: file.path, revision: file.revision, contentBase64: Buffer.from(file.bytes).toString("base64") };
  }

  async writeProjectFile(path: string, contentBase64: string, expectedRevision: string | null): Promise<unknown> {
    const checkedPath = ProjectPathSchema.parse(path);
    const result = await this.files.write(checkedPath, contentBase64, expectedRevision);
    // Persisted disk state is authoritative in LUNA-07. Browser synchronization
    // happens from the immutable snapshot before compile.
    return result;
  }

  async compile(input: {
    topModule: string;
    includeDirs?: string[];
    defines?: Record<string, string>;
    mode?: "syntax_only" | "full";
  }): Promise<CompileResult> {
    return this.withSimulation(async () => {
      const snapshot = await this.files.snapshot();
      await this.syncSnapshot(snapshot.files);
      const request = CompileRequestSchema.parse({
        projectId: this.projectId,
        snapshotId: snapshot.snapshotId,
        topModule: input.topModule,
        includeDirs: input.includeDirs ?? [],
        defines: input.defines ?? {},
        mode: input.mode ?? "full",
      });
      const result = CompileResultSchema.parse(await this.success(makeRpcRequest({ op: "simulation.compile", projectId: this.projectId, params: request })));
      if (result.snapshotId !== snapshot.snapshotId || result.projectId !== this.projectId) {
        throw new BridgeError("INVALID_REQUEST", "Browser returned a compile result for a different project snapshot");
      }
      this.compileResults.set(result.compileId, result);
      return result;
    });
  }

  async runSimulation(input: {
    topModule: string;
    untilTicks: string;
    timescale: { magnitude: string; unit: "s" | "ms" | "us" | "ns" | "ps" | "fs" };
    workBudget?: { maxWallTimeMs: number; maxOutputBytes: number; maxVcdBytes: number };
  }): Promise<RunResult> {
    return this.withSimulation(async () => {
      const snapshot = await this.files.snapshot();
      await this.syncSnapshot(snapshot.files);
      const request = ExecuteRequestSchema.parse({
        projectId: this.projectId,
        snapshotId: snapshot.snapshotId,
        topModule: input.topModule,
        until: { ticks: input.untilTicks, timescale: input.timescale },
        workBudget: input.workBudget ?? { maxWallTimeMs: 30_000, maxOutputBytes: 1_048_576, maxVcdBytes: 16 * 1024 * 1024 },
      });
      const result = RunResultSchema.parse(await this.success(makeRpcRequest({ op: "simulation.execute", projectId: this.projectId, params: request })));
      if (result.snapshotId !== snapshot.snapshotId || result.projectId !== this.projectId) {
        throw new BridgeError("INVALID_REQUEST", "Browser returned a run result for a different project snapshot");
      }
      this.runResults.set(result.runId, result);
      this.runOrder.unshift(result.runId);
      return result;
    });
  }

  async getDiagnostics(input: { compileId?: string; runId?: string }): Promise<unknown> {
    if (input.compileId || input.runId) {
      const remote = await this.success(makeRpcRequest({
        op: "simulation.get_diagnostics",
        projectId: this.projectId,
        params: {
          ...(input.compileId === undefined ? {} : { compileId: input.compileId }),
          ...(input.runId === undefined ? {} : { runId: input.runId }),
        },
      }));
      return RpcResultSchemas["simulation.get_diagnostics"].parse(remote);
    }
    const diagnostics: Diagnostic[] = [];
    for (const result of this.compileResults.values()) diagnostics.push(...result.diagnostics);
    for (const result of this.runResults.values()) diagnostics.push(...result.diagnostics);
    return { diagnostics: diagnostics.map((diagnostic) => DiagnosticSchema.parse(diagnostic)) };
  }

  async listRuns(limit: number): Promise<unknown> {
    const remote = await this.success(makeRpcRequest({ op: "simulation.list_runs", projectId: this.projectId, params: { limit } }));
    return RpcResultSchemas["simulation.list_runs"].parse(remote);
  }

  async showWaveforms(input: { artifactId: string; signalPaths?: string[]; cursor?: { ticks: string } }): Promise<unknown> {
    const remote = await this.success(makeRpcRequest({
      op: "waveform.show",
      projectId: this.projectId,
      params: {
        artifactId: input.artifactId,
        ...(input.signalPaths === undefined ? {} : { signalPaths: input.signalPaths }),
        ...(input.cursor === undefined ? {} : { cursor: input.cursor }),
      },
    }));
    return RpcResultSchemas["waveform.show"].parse(remote);
  }

  async getSimulationLog(runId: string, tailBytes: number): Promise<unknown> {
    const remote = await this.success(makeRpcRequest({ op: "simulation.get_log", projectId: this.projectId, params: { runId, tailBytes } }));
    return RpcResultSchemas["simulation.get_log"].parse(remote);
  }

  private async syncSnapshot(files: Array<{ path: string; revision: string; bytes: Uint8Array }>): Promise<void> {
    const listed = await this.success(makeRpcRequest({ op: "workspace.list_files", projectId: this.projectId, params: {} }));
    const remote = RpcResultSchemas["workspace.list_files"].parse(listed).files;
    const remoteByPath = new Map(remote.map((file) => [file.path, file]));
    const localPaths = new Set(files.map((file) => file.path));
    for (const file of files) {
      const remoteFile = remoteByPath.get(file.path);
      if (remoteFile?.revision === file.revision) continue;
      await this.success(makeRpcRequest({
        op: "workspace.write_file",
        projectId: this.projectId,
        params: {
          path: file.path,
          expectedRevision: remoteFile?.revision ?? null,
          contentBase64: Buffer.from(file.bytes).toString("base64"),
        },
      }));
    }
    for (const file of remote) {
      if (!localPaths.has(file.path)) {
        await this.success(makeRpcRequest({
          op: "workspace.delete_file",
          projectId: this.projectId,
          params: { path: file.path, expectedRevision: file.revision },
        }));
      }
    }
  }

  private async success(request: ReturnType<typeof makeRpcRequest>): Promise<unknown> {
    const response = await this.bridge.request(request);
    if (!response.ok) throw new BridgeError(response.error.code, response.error.message, response.error.retryable);
    return response.result;
  }

  private async withSimulation<T>(operation: () => Promise<T>): Promise<T> {
    if (this.simulationBusy) throw new BridgeError("SIMULATION_BUSY", "A compile or simulation request is already running", true);
    this.simulationBusy = true;
    try {
      return await operation();
    } finally {
      this.simulationBusy = false;
    }
  }
}
