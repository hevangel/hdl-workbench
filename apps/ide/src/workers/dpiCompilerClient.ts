import type { DpiCompilerFile, DpiCompilerRequest, DpiCompilerResponse } from "./dpiCompilerProtocol.js";

export interface DpiCompilerWorkerOptions {
  timeoutMs?: number;
  createWorker?: () => Worker;
}

const DEFAULT_TIMEOUT_MS = 180_000;
const MAX_TIMEOUT_MS = 300_000;

export function compileDpiProjectInBrowser(
  files: DpiCompilerFile[],
  options: DpiCompilerWorkerOptions = {},
): Promise<DpiCompilerResponse> {
  const timeoutMs = Math.min(Math.max(options.timeoutMs ?? DEFAULT_TIMEOUT_MS, 1), MAX_TIMEOUT_MS);
  const worker = options.createWorker?.()
    ?? new Worker(new URL("./dpiCompiler.worker.ts", import.meta.url), { type: "module" });
  const id = crypto.randomUUID();
  const request: DpiCompilerRequest = { id, files };

  return new Promise((resolve) => {
    let settled = false;
    const finish = (response: DpiCompilerResponse) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      worker.terminate();
      resolve(response);
    };
    const timer = setTimeout(() => {
      finish({ id, ok: false, error: { code: "TIMEOUT", message: `Browser C/C++ compilation exceeded ${timeoutMs} ms.` } });
    }, timeoutMs);

    worker.onmessage = (event: MessageEvent<DpiCompilerResponse>) => {
      if (event.data?.id !== id) return;
      finish(event.data);
    };
    worker.onerror = (event) => {
      event.preventDefault();
      finish({ id, ok: false, error: { code: "INTERNAL_ERROR", message: event.message || "Browser C/C++ compiler worker failed." } });
    };
    worker.onmessageerror = () => {
      finish({ id, ok: false, error: { code: "INTERNAL_ERROR", message: "Browser C/C++ compiler worker returned an unreadable module." } });
    };
    worker.postMessage(request);
  });
}
