import type {
  CompileAndRunProjectRequest,
  CompileAndRunRequest,
  WorkerRequest,
  WorkerResponse,
} from "./xezimProtocol";

export interface XezimWorkerOptions {
  timeoutMs?: number;
  createWorker?: () => Worker;
}

const DEFAULT_TIMEOUT_MS = 30_000;
const MAX_TIMEOUT_MS = 300_000;

export function runXezimWorker(
  input: Omit<CompileAndRunRequest, "type" | "id">,
  options: XezimWorkerOptions = {},
): Promise<WorkerResponse> {
  return launchWorker((id) => ({ ...input, type: "compileAndRun", id }), options);
}

export function runXezimProjectWorker(
  input: Omit<CompileAndRunProjectRequest, "type" | "id">,
  options: XezimWorkerOptions = {},
): Promise<WorkerResponse> {
  return launchWorker((id) => ({ ...input, type: "compileAndRunProject", id }), options);
}

function launchWorker(
  buildRequest: (id: string) => WorkerRequest,
  options: XezimWorkerOptions,
): Promise<WorkerResponse> {
  const timeoutMs = Math.min(Math.max(options.timeoutMs ?? DEFAULT_TIMEOUT_MS, 1), MAX_TIMEOUT_MS);
  const worker = options.createWorker?.() ?? new Worker(new URL("./xezim.worker.ts", import.meta.url), { type: "module" });
  const id = crypto.randomUUID();

  return new Promise((resolve) => {
    let settled = false;
    const finish = (response: WorkerResponse) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      worker.terminate();
      resolve(response);
    };
    const timer = setTimeout(() => {
      finish({ id, ok: false, error: { code: "TIMEOUT", message: `Xezim exceeded ${timeoutMs} ms.` } });
    }, timeoutMs);

    worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      if (event.data?.id !== id) return;
      finish(event.data);
    };
    worker.onerror = (event) => {
      event.preventDefault();
      finish({ id, ok: false, error: { code: "INTERNAL_ERROR", message: event.message || "Xezim worker crashed." } });
    };
    worker.onmessageerror = () => {
      finish({ id, ok: false, error: { code: "INTERNAL_ERROR", message: "Xezim worker returned an unreadable message." } });
    };
    const request = buildRequest(id);
    const transfers = request.dpiModules?.map(({ bytes }) => bytes) ?? [];
    worker.postMessage(request, transfers);
  });
}
