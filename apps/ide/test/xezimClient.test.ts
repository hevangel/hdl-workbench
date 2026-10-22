import { describe, expect, it, vi } from "vitest";
import { runXezimProjectWorker, runXezimWorker } from "../src/workers/xezimClient";
import type { WorkerRequest, WorkerResponse } from "../src/workers/xezimProtocol";

class FakeWorker {
  onmessage: ((event: MessageEvent<WorkerResponse>) => void) | null = null;
  onerror: ((event: ErrorEvent) => void) | null = null;
  onmessageerror: ((event: MessageEvent) => void) | null = null;
  posted?: WorkerRequest;
  terminated = false;

  postMessage(message: WorkerRequest) {
    this.posted = message;
  }

  terminate() {
    this.terminated = true;
  }

  respond(response: WorkerResponse) {
    this.onmessage?.({ data: response } as MessageEvent<WorkerResponse>);
  }

  crash(message: string) {
    this.onerror?.({ message, preventDefault: vi.fn() } as unknown as ErrorEvent);
  }
}

const completedResult: Extract<WorkerResponse, { ok: true }> = {
  id: "pending",
  ok: true,
  result: {
    schemaVersion: 1,
    status: "completed",
    maxTimeNs: "50",
    lastTimeTicks: "42",
    finished: true,
    sawFatal: false,
    stdout: "COUNT=4",
    diagnostics: [],
  },
};

describe("Xezim Worker client", () => {
  it("correlates the one-shot result and terminates its Worker", async () => {
    const worker = new FakeWorker();
    const pending = runXezimWorker(
      { source: "module counter; endmodule", topModule: "counter", maxTimeNs: 50 },
      { createWorker: () => worker as unknown as Worker },
    );
    const id = worker.posted?.id;
    expect(id).toBeTruthy();

    worker.respond({ ...completedResult, id: "stale" });
    worker.respond({ ...completedResult, id: id! });

    await expect(pending).resolves.toEqual({ ...completedResult, id });
    expect(worker.terminated).toBe(true);
  });

  it("posts a project snapshot request and returns its VCD metadata", async () => {
    const worker = new FakeWorker();
    const pending = runXezimProjectWorker(
      {
        files: [{ path: "rtl/top.sv", contents: "module top; endmodule" }],
        snapshotId: "sha256:revision",
        topModule: "top",
        maxTimeNs: 20,
        maxVcdBytes: 1024,
      },
      { createWorker: () => worker as unknown as Worker },
    );
    const id = worker.posted?.id;
    expect(worker.posted).toMatchObject({ type: "compileAndRunProject", snapshotId: "sha256:revision" });
    worker.respond({
      id: id!,
      ok: true,
      result: {
        ...completedResult.result,
        snapshotId: "sha256:revision",
        waveform: { format: "vcd", sizeBytes: 80, complete: true, timescale: "1ps" },
        vcd: "$enddefinitions $end",
      },
    });
    await expect(pending).resolves.toMatchObject({
      ok: true,
      result: { snapshotId: "sha256:revision", waveform: { sizeBytes: 80 }, vcd: "$enddefinitions $end" },
    });
    expect(worker.terminated).toBe(true);
  });

  it("times out a stuck worker and can start a fresh worker afterward", async () => {
    vi.useFakeTimers();
    const workers: FakeWorker[] = [];
    const createWorker = () => {
      const worker = new FakeWorker();
      workers.push(worker);
      return worker as unknown as Worker;
    };

    const timedOut = runXezimWorker(
      { source: "", topModule: "counter", maxTimeNs: 1 },
      { timeoutMs: 20, createWorker },
    );
    await vi.advanceTimersByTimeAsync(20);
    await expect(timedOut).resolves.toMatchObject({
      ok: false,
      error: { code: "TIMEOUT" },
    });
    expect(workers[0]?.terminated).toBe(true);

    const recovered = runXezimWorker(
      { source: "", topModule: "counter", maxTimeNs: 1 },
      { createWorker },
    );
    const next = workers[1]!;
    next.respond({ ...completedResult, id: next.posted!.id });
    await expect(recovered).resolves.toMatchObject({ ok: true, result: { stdout: "COUNT=4" } });
    vi.useRealTimers();
  });

  it("converts a worker crash into an error and permits a later request", async () => {
    const workers: FakeWorker[] = [];
    const createWorker = () => {
      const worker = new FakeWorker();
      workers.push(worker);
      return worker as unknown as Worker;
    };
    const crashed = runXezimWorker(
      { source: "", topModule: "counter", maxTimeNs: 1 },
      { createWorker },
    );
    workers[0]!.crash("wasm worker trapped");
    await expect(crashed).resolves.toMatchObject({
      ok: false,
      error: { code: "INTERNAL_ERROR", message: "wasm worker trapped" },
    });
    expect(workers[0]?.terminated).toBe(true);

    const recovered = runXezimWorker(
      { source: "", topModule: "counter", maxTimeNs: 1 },
      { createWorker },
    );
    workers[1]!.respond({ ...completedResult, id: workers[1]!.posted!.id });
    await expect(recovered).resolves.toMatchObject({ ok: true });
  });
});
