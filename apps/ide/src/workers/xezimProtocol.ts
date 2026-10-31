export interface CompileAndRunRequest {
  type: "compileAndRun";
  id: string;
  source: string;
  dpiModules?: DpiWasmModuleInput[];
  topModule: string;
  maxTimeNs: number;
  maxOutputBytes?: number;
}

export interface ProjectFileInput {
  path: string;
  contents: string;
}

export interface DpiWasmModuleInput {
  path: string;
  bytes: ArrayBuffer;
}

export interface CompileAndRunProjectRequest {
  type: "compileAndRunProject";
  id: string;
  files: ProjectFileInput[];
  dpiModules?: DpiWasmModuleInput[];
  snapshotId: string;
  topModule: string;
  maxTimeNs: number;
  maxOutputBytes?: number;
  maxVcdBytes?: number;
  /** `+name` or `+name=value` arguments for $test$plusargs and $value$plusargs. */
  plusargs?: string[];
  waveformFormat?: "vcd" | "fst";
}

export type WorkerRequest = CompileAndRunRequest | CompileAndRunProjectRequest;

export interface WorkerResult {
  schemaVersion: 1;
  status: "completed" | "failed" | "resource_limit";
  maxTimeNs: string;
  lastTimeTicks: string;
  finished: boolean;
  sawFatal: boolean;
  stdout: string;
  diagnostics: Array<{ severity: "error" | "warning"; code: string; message: string }>;
  snapshotId?: string;
  waveform?: { format: "vcd" | "fst"; sizeBytes: number; complete: boolean; timescale?: string | null };
  vcd?: string;
  /** The FST file, base64-encoded, when `waveformFormat` was "fst". */
  fstBase64?: string;
  fileOutputs?: Array<{ path: string; contents: string }>;
}

export type WorkerResponse =
  | { id: string; ok: true; result: WorkerResult }
  | { id: string; ok: false; error: { code: string; message: string } };
