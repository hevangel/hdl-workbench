import type { CompileRequest, CompileResult, RpcResponse, RunRequest, RunResult } from "./contracts.js";
import { DecimalTicksSchema, RpcResultSchemas } from "./schemas.js";
import type { RpcRequest } from "./contracts.js";

export class ContractValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ContractValidationError";
  }
}

/**
 * Compare normalized project paths by UTF-16 code units so Node and browsers
 * produce the same order regardless of the user's locale.
 */
export function compareProjectPaths(left: string, right: string): -1 | 0 | 1 {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function validateCompileResultForRequest(request: CompileRequest, result: CompileResult): void {
  if (result.projectId !== request.projectId || result.snapshotId !== request.snapshotId) {
    throw new ContractValidationError("Compile result provenance does not match the requested project snapshot.");
  }
}

export function validateRunResultForRequest(request: Pick<RunRequest, "projectId" | "snapshotId">, result: RunResult): void {
  if (result.projectId !== request.projectId || result.snapshotId !== request.snapshotId) {
    throw new ContractValidationError("Run result provenance does not match the requested project snapshot.");
  }
  if (result.waveform && (result.waveform.runId !== result.runId || result.waveform.snapshotId !== request.snapshotId)) {
    throw new ContractValidationError("Waveform provenance does not match the run result.");
  }
}

export function validateRpcRequestConsistency(request: RpcRequest): void {
  if (
    (request.op === "simulation.compile" || request.op === "simulation.run" || request.op === "simulation.execute") &&
    request.projectId !== request.params.projectId
  ) {
    throw new ContractValidationError("RPC projectId does not match the project in the operation parameters.");
  }
}

export function validateRpcResponseForRequest(request: RpcRequest, response: RpcResponse): void {
  validateRpcRequestConsistency(request);
  if (response.protocol !== request.protocol || response.id !== request.id) {
    throw new ContractValidationError("RPC response correlation does not match the request.");
  }
  if (!response.ok) return;

  const parsed = RpcResultSchemas[request.op].safeParse(response.result);
  if (!parsed.success) {
    throw new ContractValidationError(`RPC result does not match operation ${request.op}.`);
  }
  if (request.op === "simulation.compile") {
    validateCompileResultForRequest(request.params, parsed.data as CompileResult);
  } else if (request.op === "simulation.run") {
    validateRunResultForRequest(request.params, parsed.data as RunResult);
  } else if (request.op === "simulation.execute") {
    validateRunResultForRequest(request.params, parsed.data as RunResult);
  } else if (request.op === "workspace.read_file" && (parsed.data as unknown as { path: string }).path !== request.params.path) {
    throw new ContractValidationError("File response path does not match the requested path.");
  }
}

export function serializeTicks(value: bigint): string {
  if (value < 0n) throw new ContractValidationError("Simulation ticks must be nonnegative.");
  const wireValue = value.toString(10);
  if (!DecimalTicksSchema.safeParse(wireValue).success) {
    throw new ContractValidationError("Simulation tick value exceeds the v1 wire limit.");
  }
  return wireValue;
}

export function parseTicks(value: string): bigint {
  if (!DecimalTicksSchema.safeParse(value).success) {
    throw new ContractValidationError("Simulation ticks must be a bounded canonical decimal string.");
  }
  return BigInt(value);
}

export function compareTicks(left: string, right: string): -1 | 0 | 1 {
  const a = parseTicks(left);
  const b = parseTicks(right);
  return a < b ? -1 : a > b ? 1 : 0;
}

export function isValidTimeRange(startTicks: string, endTicks: string): boolean {
  return compareTicks(startTicks, endTicks) <= 0;
}
