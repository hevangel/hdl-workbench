import type { WorkerRequest, WorkerResponse } from "./xezimProtocol";

interface WorkerScope {
  location: Location;
  onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
  postMessage(message: WorkerResponse): void;
}

const workerScope = globalThis as unknown as WorkerScope;

const MAX_SOURCE_BYTES = 4 * 1024 * 1024;
const MAX_TIME_NS = 10_000_000;
const MAX_OUTPUT_BYTES = 1024 * 1024;

type XezimModule = {
  default: () => Promise<unknown>;
  compileAndRun: (source: string, topModule: string, maxTimeNs: bigint, maxOutputBytes: number) => string;
};

let modulePromise: Promise<XezimModule> | undefined;

function loadXezim(): Promise<XezimModule> {
  modulePromise ??= (async () => {
    const wasmModuleUrl = new URL("/wasm/xezim_web.js", workerScope.location.href);
    const wasm = (await import(/* @vite-ignore */ wasmModuleUrl.href)) as XezimModule;
    await wasm.default();
    return wasm;
  })();
  return modulePromise;
}

function fail(id: string, code: string, message: string): WorkerResponse {
  return { id, ok: false, error: { code, message } };
}

workerScope.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const request = event.data;
  if (request?.type !== "compileAndRun" || typeof request.id !== "string") return;
  try {
    if (typeof request.source !== "string") {
      workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "Source must be a string."));
      return;
    }
    if (new TextEncoder().encode(request.source).byteLength > MAX_SOURCE_BYTES) {
      workerScope.postMessage(fail(request.id, "RESOURCE_LIMIT", "Source exceeds the 4 MiB browser limit."));
      return;
    }
    if (typeof request.topModule !== "string" || !request.topModule || request.topModule.length > 512) {
      workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "A top module of at most 512 characters is required."));
      return;
    }
    if (!Number.isSafeInteger(request.maxTimeNs) || request.maxTimeNs < 0 || request.maxTimeNs > MAX_TIME_NS) {
      workerScope.postMessage(fail(request.id, "RESOURCE_LIMIT", "Simulation time must be an integer from 0 to 10000000 ns."));
      return;
    }
    if (request.maxOutputBytes !== undefined
      && (!Number.isSafeInteger(request.maxOutputBytes) || request.maxOutputBytes < 0)) {
      workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "Output limit must be a nonnegative integer."));
      return;
    }
    const wasm = await loadXezim();
    const json = wasm.compileAndRun(
      request.source,
      request.topModule,
      BigInt(request.maxTimeNs),
      Math.min(request.maxOutputBytes ?? MAX_OUTPUT_BYTES, MAX_OUTPUT_BYTES),
    );
    const result = JSON.parse(json) as WorkerResponse extends { ok: true; result: infer T } ? T : never;
    workerScope.postMessage({ id: request.id, ok: true, result } satisfies WorkerResponse);
  } catch (error) {
    workerScope.postMessage(fail(
      request.id,
      "INTERNAL_ERROR",
      error instanceof Error ? error.message.slice(0, 4096) : "Xezim worker failed.",
    ));
  }
};
