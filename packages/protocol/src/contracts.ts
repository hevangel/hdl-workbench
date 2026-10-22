import type {
  CompileRequestSchema,
  CompileResultSchema,
  CompileIdSchema,
  DiagnosticSchema,
  ArtifactIdSchema,
  ConnectionStatusSchema,
  FileEntrySchema,
  ProjectIdSchema,
  ProjectPathSchema,
  RevisionSchema,
  RunIdSchema,
  RpcRequestSchema,
  RunRequestSchema,
  RunResultSchema,
  RpcResponseSchema,
  SnapshotIdSchema,
  WaveformTimeSchema,
  WaveformArtifactSchema,
} from "./schemas.js";
import { z } from "zod";

export type ProjectPath = z.infer<typeof ProjectPathSchema>;
export type ProjectId = z.infer<typeof ProjectIdSchema>;
export type Revision = z.infer<typeof RevisionSchema>;
export type FileEntry = z.infer<typeof FileEntrySchema>;
export type SnapshotId = z.infer<typeof SnapshotIdSchema>;
export type CompileId = z.infer<typeof CompileIdSchema>;
export type RunId = z.infer<typeof RunIdSchema>;
export type ArtifactId = z.infer<typeof ArtifactIdSchema>;
export type RpcRequest = z.infer<typeof RpcRequestSchema>;
export type RpcResponse = z.infer<typeof RpcResponseSchema>;
export type ConnectionStatus = z.infer<typeof ConnectionStatusSchema>;
export type WaveformTime = z.infer<typeof WaveformTimeSchema>;
export type Diagnostic = z.infer<typeof DiagnosticSchema>;
export type CompileRequest = z.infer<typeof CompileRequestSchema>;
export type CompileResult = z.infer<typeof CompileResultSchema>;
export type RunRequest = z.infer<typeof RunRequestSchema>;
export type RunResult = z.infer<typeof RunResultSchema>;
export type WaveformArtifact = z.infer<typeof WaveformArtifactSchema>;

export interface SnapshotFile {
  path: ProjectPath;
  revision: Revision;
  bytes: Uint8Array;
}

export interface FileChange {
  path: ProjectPath;
  revision: Revision | null;
  operation: "write" | "delete";
}

export interface WorkspaceAdapter {
  listFiles(): Promise<FileEntry[]>;
  readFile(path: ProjectPath): Promise<{ bytes: Uint8Array; revision: Revision }>;
  writeFile(path: ProjectPath, bytes: Uint8Array, expectedRevision: Revision | null): Promise<{ revision: Revision }>;
  deleteFile(path: ProjectPath, expectedRevision: Revision): Promise<void>;
  snapshot(): Promise<{ snapshotId: SnapshotId; files: SnapshotFile[] }>;
  subscribeChanges(callback: (change: FileChange) => void): () => void;
}

export interface SimulationService {
  compile(input: CompileRequest): Promise<CompileResult>;
  run(input: RunRequest): Promise<RunResult>;
}

export interface SurferAdapter {
  ready(): Promise<void>;
  loadWaveform(artifact: WaveformArtifact): Promise<void>;
  addSignals(paths: string[]): Promise<void>;
  setCursor(time: WaveformTime): Promise<void>;
  zoom(start: WaveformTime, end: WaveformTime): Promise<void>;
  dispose(): void;
}

export interface IdeController {
  connectionStatus(): Promise<ConnectionStatus>;
  listFiles(): Promise<FileEntry[]>;
  readFile(path: ProjectPath): Promise<{ bytes: Uint8Array; revision: Revision }>;
  writeFile(path: ProjectPath, bytes: Uint8Array, expectedRevision: Revision | null): Promise<{ revision: Revision }>;
  deleteFile(path: ProjectPath, expectedRevision: Revision): Promise<void>;
  compile(input: CompileRequest): Promise<CompileResult>;
  run(input: RunRequest): Promise<RunResult>;
  diagnostics(input: { compileId?: CompileId; runId?: RunId }): Promise<Diagnostic[]>;
  listRuns(projectId: ProjectId, limit: number): Promise<RunResult[]>;
  showWaveform(input: { artifactId: ArtifactId; signalPaths?: string[]; cursor?: WaveformTime }): Promise<void>;
  getLog(input: { runId: RunId; tailBytes: number }): Promise<{ stdout: string; stderr: string }>;
}
