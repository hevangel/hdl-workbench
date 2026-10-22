export interface WorkerRequest {
  type: "compileAndRun";
  id: string;
  source: string;
  topModule: string;
  maxTimeNs: number;
  maxOutputBytes?: number;
}

export interface WorkerResult {
  schemaVersion: 1;
  status: "completed" | "failed" | "resource_limit";
  maxTimeNs: string;
  lastTimeTicks: string;
  finished: boolean;
  sawFatal: boolean;
  stdout: string;
  diagnostics: Array<{ severity: "error" | "warning"; code: string; message: string }>;
}

export type WorkerResponse =
  | { id: string; ok: true; result: WorkerResult }
  | { id: string; ok: false; error: { code: string; message: string } };
