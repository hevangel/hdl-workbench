import { z } from "zod";

export const PROTOCOL_VERSION = "xezim-ide/1" as const;
export const ProtocolVersionSchema = z.literal(PROTOCOL_VERSION);

const idPattern = /^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$/;
export const ProjectIdSchema = z.string().regex(idPattern);
export const SnapshotIdSchema = z.string().regex(idPattern);
export const CompileIdSchema = z.string().regex(idPattern);
export const RunIdSchema = z.string().regex(idPattern);
export const ArtifactIdSchema = z.string().regex(idPattern);
export const RequestIdSchema = z.string().regex(idPattern);
export const RevisionSchema = z.string().min(1).max(256);

// Paths are project-relative POSIX paths. Keep validation regex-based so the
// same constraint can be emitted in JSON Schema for non-TypeScript clients.
export const ProjectPathSchema = z
  .string()
  .min(1)
  .max(1024)
  .regex(/^(?!\/)(?![A-Za-z]:)(?!.*(?:^|\/)\.{1,2}(?:\/|$))(?!.*\/\/)(?!.*[\\\u0000-\u001f\u007f])[^/]+(?:\/[^/]+)*$/);

export const DecimalTicksSchema = z.string().max(40).regex(/^(0|[1-9][0-9]*)$/);
export const TimeUnitSchema = z.enum(["s", "ms", "us", "ns", "ps", "fs"]);
export const TimescaleSchema = z.strictObject({
  magnitude: z.string().regex(/^[1-9][0-9]{0,2}$/),
  unit: TimeUnitSchema,
});
export const WaveformTimeSchema = z.strictObject({ ticks: DecimalTicksSchema });

export const FileEntrySchema = z.strictObject({
  path: ProjectPathSchema,
  revision: RevisionSchema,
  sizeBytes: z.number().int().nonnegative().max(16 * 1024 * 1024),
});

const base64Pattern = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;
export const ProjectFileSchema = z.strictObject({
  path: ProjectPathSchema,
  revision: RevisionSchema,
  contentBase64: z.string().max(22_369_624).regex(base64Pattern),
});
export const ProjectSnapshotSchema = z.strictObject({
  projectId: ProjectIdSchema,
  snapshotId: SnapshotIdSchema,
  files: z.array(ProjectFileSchema).max(10_000),
});

export const DiagnosticSchema = z.strictObject({
  severity: z.enum(["error", "warning", "info"]),
  code: z.string().min(1).max(128),
  message: z.string().min(1).max(16_384),
  path: ProjectPathSchema.optional(),
  line: z.number().int().positive().optional(),
  column: z.number().int().positive().optional(),
});

export const CompileRequestSchema = z.strictObject({
  projectId: ProjectIdSchema,
  snapshotId: SnapshotIdSchema,
  topModule: z.string().min(1).max(512),
  includeDirs: z.array(ProjectPathSchema).max(256),
  defines: z.record(z.string(), z.string().max(4096)).default({}),
  mode: z.enum(["syntax_only", "full"]),
});
export const CompileResultSchema = z.strictObject({
  compileId: CompileIdSchema,
  projectId: ProjectIdSchema,
  snapshotId: SnapshotIdSchema,
  success: z.boolean(),
  diagnostics: z.array(DiagnosticSchema).max(100_000),
});

export const WorkBudgetSchema = z.strictObject({
  maxSteps: z.number().int().positive().max(100_000_000),
  maxWallTimeMs: z.number().int().positive().max(300_000),
  maxOutputBytes: z.number().int().nonnegative().max(16 * 1024 * 1024),
  maxVcdBytes: z.number().int().nonnegative().max(256 * 1024 * 1024),
});
export const RunRequestSchema = z.strictObject({
  projectId: ProjectIdSchema,
  snapshotId: SnapshotIdSchema,
  compileId: CompileIdSchema,
  until: z.strictObject({ ticks: DecimalTicksSchema, timescale: TimescaleSchema }),
  workBudget: WorkBudgetSchema,
  vcdScopes: z.array(z.string().min(1).max(1024)).max(4096).optional(),
});

export const WaveformArtifactSchema = z.strictObject({
  id: ArtifactIdSchema,
  runId: RunIdSchema,
  snapshotId: SnapshotIdSchema,
  format: z.literal("vcd"),
  sizeBytes: z.number().int().nonnegative().max(256 * 1024 * 1024),
  storage: z.enum(["opfs", "bridge-file"]),
  timescale: TimescaleSchema,
  complete: z.boolean(),
});

export const RunStatusSchema = z.enum(["completed", "finish", "failed", "timeout", "resource_limit"]);
export const RunResultSchema = z.strictObject({
  runId: RunIdSchema,
  projectId: ProjectIdSchema,
  snapshotId: SnapshotIdSchema,
  status: RunStatusSchema,
  diagnostics: z.array(DiagnosticSchema).max(100_000),
  stdout: z.string().max(16 * 1024 * 1024),
  stderr: z.string().max(16 * 1024 * 1024),
  lastTime: z.strictObject({ ticks: DecimalTicksSchema, timescale: TimescaleSchema }),
  waveform: WaveformArtifactSchema.optional(),
});

export const ErrorCodeSchema = z.enum([
  "UNSUPPORTED_CAPABILITY",
  "SIMULATOR_UNAVAILABLE",
  "BROWSER_DISCONNECTED",
  "REVISION_CONFLICT",
  "INVALID_PATH",
  "SIMULATION_BUSY",
  "QUOTA_EXCEEDED",
  "UNAUTHORIZED",
  "NOT_FOUND",
  "TIMEOUT",
  "RESOURCE_LIMIT",
  "INVALID_REQUEST",
  "INTERNAL_ERROR",
]);
export const RpcErrorSchema = z.strictObject({
  code: ErrorCodeSchema,
  message: z.string().min(1).max(4096),
  retryable: z.boolean().optional(),
});

const EmptyParamsSchema = z.strictObject({});
const ListFilesParamsSchema = z.strictObject({});
const ReadFileParamsSchema = z.strictObject({ path: ProjectPathSchema });
const WriteFileParamsSchema = z.strictObject({
  path: ProjectPathSchema,
  expectedRevision: RevisionSchema.nullable(),
  contentBase64: ProjectFileSchema.shape.contentBase64,
});
const DeleteFileParamsSchema = z.strictObject({ path: ProjectPathSchema, expectedRevision: RevisionSchema });
const GetDiagnosticsParamsSchema = z.strictObject({ compileId: CompileIdSchema.optional(), runId: RunIdSchema.optional() });
const ListRunsParamsSchema = z.strictObject({ limit: z.number().int().positive().max(100).default(20) });
const ShowWaveformParamsSchema = z.strictObject({
  artifactId: ArtifactIdSchema,
  signalPaths: z.array(z.string().min(1).max(1024)).max(4096).optional(),
  cursor: WaveformTimeSchema.optional(),
});
const GetLogParamsSchema = z.strictObject({ runId: RunIdSchema, tailBytes: z.number().int().positive().max(1_048_576).default(65_536) });

export const RpcRequestSchema = z.discriminatedUnion("op", [
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("connection.get_status"), projectId: ProjectIdSchema, params: EmptyParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("workspace.list_files"), projectId: ProjectIdSchema, params: ListFilesParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("workspace.read_file"), projectId: ProjectIdSchema, params: ReadFileParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("workspace.write_file"), projectId: ProjectIdSchema, params: WriteFileParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("workspace.delete_file"), projectId: ProjectIdSchema, params: DeleteFileParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("simulation.compile"), projectId: ProjectIdSchema, params: CompileRequestSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("simulation.run"), projectId: ProjectIdSchema, params: RunRequestSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("simulation.get_diagnostics"), projectId: ProjectIdSchema, params: GetDiagnosticsParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("simulation.list_runs"), projectId: ProjectIdSchema, params: ListRunsParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("simulation.get_log"), projectId: ProjectIdSchema, params: GetLogParamsSchema }),
  z.strictObject({ protocol: ProtocolVersionSchema, id: RequestIdSchema, op: z.literal("waveform.show"), projectId: ProjectIdSchema, params: ShowWaveformParamsSchema }),
]);

export const PairingChallengeSchema = z.strictObject({
  challengeId: RequestIdSchema,
  code: z.string().regex(/^[A-Z2-7]{8,16}$/),
  expiresAt: z.string().datetime({ offset: true }),
});
export const PairingRequestSchema = z.strictObject({
  challengeId: RequestIdSchema,
  code: z.string().regex(/^[A-Z2-7]{8,16}$/),
  clientName: z.string().min(1).max(128),
  requestedRole: z.enum(["read", "write"]),
});
export const PairingResultSchema = z.strictObject({
  sessionToken: z.string().min(32).max(512),
  role: z.enum(["read", "write"]),
  expiresAt: z.string().datetime({ offset: true }),
});

export const CapabilityNameSchema = z.enum(["browser_simulation", "waveform_load", "waveform_add_signals", "waveform_cursor"]);
export const CapabilitySchema = z.strictObject({
  name: CapabilityNameSchema,
  available: z.boolean(),
  reason: z.string().min(1).max(1024).optional(),
});
export const ConnectionStatusSchema = z.strictObject({
  protocol: ProtocolVersionSchema,
  connected: z.boolean(),
  capabilities: z.array(CapabilitySchema),
});

export const RpcResultSchemas = {
  "connection.get_status": ConnectionStatusSchema,
  "workspace.list_files": z.strictObject({ files: z.array(FileEntrySchema).max(10_000) }),
  "workspace.read_file": ProjectFileSchema,
  "workspace.write_file": z.strictObject({ revision: RevisionSchema }),
  "workspace.delete_file": z.strictObject({ deleted: z.literal(true) }),
  "simulation.compile": CompileResultSchema,
  "simulation.run": RunResultSchema,
  "simulation.get_diagnostics": z.strictObject({ diagnostics: z.array(DiagnosticSchema).max(100_000) }),
  "simulation.list_runs": z.strictObject({ runs: z.array(RunResultSchema).max(100), nextCursor: z.string().min(1).max(512).optional() }),
  "simulation.get_log": z.strictObject({ runId: RunIdSchema, stdout: z.string().max(1_048_576), stderr: z.string().max(1_048_576) }),
  "waveform.show": z.strictObject({ artifactId: ArtifactIdSchema, opened: z.literal(true) }),
} as const;
export const RpcResultSchema = z.union([
  ConnectionStatusSchema,
  RpcResultSchemas["workspace.list_files"],
  RpcResultSchemas["workspace.read_file"],
  RpcResultSchemas["workspace.write_file"],
  RpcResultSchemas["workspace.delete_file"],
  CompileResultSchema,
  RunResultSchema,
  RpcResultSchemas["simulation.get_diagnostics"],
  RpcResultSchemas["simulation.list_runs"],
  RpcResultSchemas["simulation.get_log"],
  RpcResultSchemas["waveform.show"],
]);
export const RpcSuccessSchema = z.strictObject({
  protocol: ProtocolVersionSchema,
  id: RequestIdSchema,
  ok: z.literal(true),
  result: RpcResultSchema,
});
export const RpcFailureSchema = z.strictObject({
  protocol: ProtocolVersionSchema,
  id: RequestIdSchema,
  ok: z.literal(false),
  error: RpcErrorSchema,
});
export const RpcResponseSchema = z.discriminatedUnion("ok", [RpcSuccessSchema, RpcFailureSchema]);

export const WireSchemas = {
  protocolVersion: ProtocolVersionSchema,
  projectId: ProjectIdSchema,
  snapshotId: SnapshotIdSchema,
  revision: RevisionSchema,
  projectPath: ProjectPathSchema,
  decimalTicks: DecimalTicksSchema,
  timescale: TimescaleSchema,
  waveformTime: WaveformTimeSchema,
  fileEntry: FileEntrySchema,
  projectFile: ProjectFileSchema,
  projectSnapshot: ProjectSnapshotSchema,
  diagnostic: DiagnosticSchema,
  compileRequest: CompileRequestSchema,
  compileResult: CompileResultSchema,
  runRequest: RunRequestSchema,
  runResult: RunResultSchema,
  waveformArtifact: WaveformArtifactSchema,
  rpcRequest: RpcRequestSchema,
  rpcResult: RpcResultSchema,
  rpcResponse: RpcResponseSchema,
  pairingChallenge: PairingChallengeSchema,
  pairingRequest: PairingRequestSchema,
  pairingResult: PairingResultSchema,
  connectionStatus: ConnectionStatusSchema,
} as const;
