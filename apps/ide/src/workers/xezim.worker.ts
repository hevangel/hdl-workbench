import type { WorkerRequest, WorkerResponse } from "./xezimProtocol";
import { DpiWasmModuleError, installDpiWasmModules } from "./dpiWasmModules.js";

interface WorkerScope {
  location: Location;
  onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
  postMessage(message: WorkerResponse): void;
}

const workerScope = globalThis as unknown as WorkerScope;

const MAX_SOURCE_BYTES = 4 * 1024 * 1024;
const MAX_PROJECT_BYTES = 16 * 1024 * 1024;
const MAX_PROJECT_FILES = 4096;
const MAX_VCD_BYTES = 64 * 1024 * 1024;
const MAX_TIME_NS = 10_000_000;
const MAX_OUTPUT_BYTES = 1024 * 1024;
const MAX_WASM_MEMORY_BYTES = 256 * 1024 * 1024;

type XezimModule = {
  default: () => Promise<{ memory: WebAssembly.Memory }>;
  compileAndRun: (source: string, topModule: string, maxTimeNs: bigint, maxOutputBytes: number) => string;
  compileAndRunProject: (
    filesJson: string,
    snapshotId: string,
    topModule: string,
    maxTimeNs: bigint,
    maxOutputBytes: number,
    maxVcdBytes: number,
  ) => string;
  compileAndRunProjectWithOptions?: (
    filesJson: string,
    snapshotId: string,
    topModule: string,
    maxTimeNs: bigint,
    maxOutputBytes: number,
    maxVcdBytes: number,
    optionsJson: string,
  ) => string;
};

let modulePromise: Promise<XezimModule> | undefined;
let wasmMemory: WebAssembly.Memory | undefined;

function loadXezim(): Promise<XezimModule> {
  modulePromise ??= (async () => {
    const wasmModuleUrl = new URL("/wasm/xezim_web.js", workerScope.location.href);
    const wasm = (await import(/* @vite-ignore */ wasmModuleUrl.href)) as XezimModule;
    const exports = await wasm.default();
    wasmMemory = exports.memory;
    return wasm;
  })();
  return modulePromise;
}

function fail(id: string, code: string, message: string): WorkerResponse {
  return { id, ok: false, error: { code, message } };
}

workerScope.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const request = event.data;
  if ((request?.type !== "compileAndRun" && request?.type !== "compileAndRunProject")
    || typeof request.id !== "string") return;
  try {
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
    let json: string;
    if (request.type === "compileAndRun") {
      if (typeof request.source !== "string") {
        workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "Source must be a string."));
        return;
      }
      if (new TextEncoder().encode(request.source).byteLength > MAX_SOURCE_BYTES) {
        workerScope.postMessage(fail(request.id, "RESOURCE_LIMIT", "Source exceeds the 4 MiB browser limit."));
        return;
      }
      const uninstallDpiDispatcher = await installDpiWasmModules(request.dpiModules ?? []);
      try {
        json = wasm.compileAndRun(
          request.source,
          request.topModule,
          BigInt(request.maxTimeNs),
          Math.min(request.maxOutputBytes ?? MAX_OUTPUT_BYTES, MAX_OUTPUT_BYTES),
        );
      } finally {
        uninstallDpiDispatcher();
      }
    } else {
      if (!Array.isArray(request.files) || request.files.length === 0 || request.files.length > MAX_PROJECT_FILES) {
        workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "Project must contain 1 to 4096 files."));
        return;
      }
      if (typeof request.snapshotId !== "string" || !request.snapshotId || request.snapshotId.length > 256) {
        workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "A snapshot ID of at most 256 characters is required."));
        return;
      }
      if (request.maxVcdBytes !== undefined
        && (!Number.isSafeInteger(request.maxVcdBytes) || request.maxVcdBytes < 0)) {
        workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "VCD limit must be a nonnegative integer."));
        return;
      }
      const encoder = new TextEncoder();
      let totalBytes = 0;
      for (const file of request.files) {
        if (!file || typeof file.path !== "string" || typeof file.contents !== "string") {
          workerScope.postMessage(fail(request.id, "INVALID_REQUEST", "Each project file needs a path and UTF-8 contents."));
          return;
        }
        const path = file.path.replaceAll("\\", "/");
        if (!path || path.startsWith("/") || /^[A-Za-z]:/.test(path) || path.split("/").includes("..")) {
          workerScope.postMessage(fail(request.id, "INVALID_PATH", `Project path is not relative: ${file.path}`));
          return;
        }
        totalBytes += encoder.encode(file.path).byteLength + encoder.encode(file.contents).byteLength;
        if (totalBytes > MAX_PROJECT_BYTES) {
          workerScope.postMessage(fail(request.id, "RESOURCE_LIMIT", "Project files exceed the 16 MiB browser limit."));
          return;
        }
      }
      const uninstallDpiDispatcher = await installDpiWasmModules(request.dpiModules ?? []);
      try {
        const wantsOptions = Boolean(request.plusargs?.length) || request.waveformFormat === "fst";
        if (wantsOptions && !wasm.compileAndRunProjectWithOptions) {
          workerScope.postMessage(fail(request.id, "UNSUPPORTED_CAPABILITY", "This Xezim WASM build predates plusargs and FST output. Rebuild it with pnpm build:browser."));
          return;
        }
        const projectArguments = [
          JSON.stringify(request.files),
          request.snapshotId,
          request.topModule,
          BigInt(request.maxTimeNs),
          Math.min(request.maxOutputBytes ?? MAX_OUTPUT_BYTES, MAX_OUTPUT_BYTES),
          Math.min(request.maxVcdBytes ?? MAX_VCD_BYTES, MAX_VCD_BYTES),
        ] as const;
        json = wantsOptions && wasm.compileAndRunProjectWithOptions
          ? wasm.compileAndRunProjectWithOptions(...projectArguments, JSON.stringify({
            plusargs: request.plusargs ?? [],
            waveformFormat: request.waveformFormat ?? "vcd",
          }))
          : wasm.compileAndRunProject(...projectArguments);
      } finally {
        uninstallDpiDispatcher();
      }
    }
    const result = JSON.parse(json) as WorkerResponse extends { ok: true; result: infer T } ? T : never;
    workerScope.postMessage({ id: request.id, ok: true, result } satisfies WorkerResponse);
  } catch (error) {
    if (error instanceof DpiWasmModuleError) {
      workerScope.postMessage(fail(request.id, error.code, error.message.slice(0, 4096)));
      return;
    }
    const wasmMemoryBytes = wasmMemory?.buffer.byteLength ?? 0;
    // Rust's allocator can trap before memory.grow reaches its declared maximum
    // when the next allocation cannot fit in the remaining address space.
    const memoryLimitReached = wasmMemoryBytes >= MAX_WASM_MEMORY_BYTES
      || (error instanceof WebAssembly.RuntimeError
        && wasmMemoryBytes >= MAX_WASM_MEMORY_BYTES - 32 * 1024 * 1024);
    workerScope.postMessage(fail(
      request.id,
      memoryLimitReached ? "RESOURCE_LIMIT" : "INTERNAL_ERROR",
      memoryLimitReached
        ? `Simulation exceeded available capacity under the 256 MiB WebAssembly memory limit (${(wasmMemoryBytes / (1024 * 1024)).toFixed(1)} MiB allocated). Reduce the design or its initialized state.`
        : error instanceof Error ? error.message.slice(0, 4096) : "Xezim worker failed.",
    ));
  }
};
