import { beforeEach, describe, expect, it, vi } from "vitest";
import { ProjectPathSchema, type ExecuteRequest } from "@xezim-ide/protocol";
import { BrowserControllerError, BrowserIdeController } from "../src/controller/BrowserIdeController.js";
import { createMockServices } from "../src/mockServices.js";
import type { WorkerResponse } from "../src/workers/xezimProtocol.js";
import type { OpfsWaveformStore } from "../src/controller/OpfsWaveformStore.js";

const vcd = "$timescale 1 ns $end\n$enddefinitions $end\n#0\n";

function request(overrides: Partial<ExecuteRequest> = {}): ExecuteRequest {
  return {
    projectId: "demo-project",
    snapshotId: "mock-snapshot-1",
    topModule: "counter_tb",
    until: { ticks: "1000", timescale: { magnitude: "1", unit: "ns" } },
    workBudget: { maxWallTimeMs: 5000, maxOutputBytes: 1_048_576, maxVcdBytes: 64 * 1024 * 1024 },
    ...overrides,
  };
}

function stableWorkspace(services: ReturnType<typeof createMockServices>) {
  return {
    ...services.workspace,
    snapshot: async () => ({ ...(await services.workspace.snapshot()), snapshotId: "mock-snapshot-1" }),
  };
}

describe("BrowserIdeController one-shot execution", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("executes the exact workspace snapshot in the Xezim Worker and persists its VCD artifact", async () => {
    const services = createMockServices();
    const stored: unknown[] = [];
    const artifacts = {
      put: vi.fn(async (metadata, text) => { stored.push({ metadata, text }); }),
      get: vi.fn(),
      objectUrl: vi.fn(),
      dispose: vi.fn(),
    } as unknown as OpfsWaveformStore;
    const worker = vi.fn(async (input): Promise<WorkerResponse> => ({
      id: "worker-1",
      ok: true,
      result: {
        schemaVersion: 1,
        status: "completed",
        maxTimeNs: "1000",
        lastTimeTicks: "1000",
        finished: true,
        sawFatal: false,
        stdout: "COUNT=4\n",
        diagnostics: [],
        snapshotId: input.snapshotId,
        waveform: { format: "vcd", sizeBytes: new TextEncoder().encode(vcd).byteLength, complete: true, timescale: "1 ns" },
        vcd,
        fileOutputs: [],
      },
    }));
    const workspace = stableWorkspace(services);
    const controller = new BrowserIdeController(workspace, { runProjectWorker: worker, artifactStore: artifacts });
    const snapshot = await workspace.snapshot();
    const result = await controller.execute(request({ snapshotId: snapshot.snapshotId }));

    expect(worker).toHaveBeenCalledWith(expect.objectContaining({
      snapshotId: snapshot.snapshotId,
      topModule: "counter_tb",
      maxTimeNs: 1000,
      files: expect.arrayContaining([expect.objectContaining({ path: "rtl/counter.sv" })]),
    }), { timeoutMs: 5000 });
    expect(result).toMatchObject({ status: "finish", stdout: "COUNT=4\n", snapshotId: snapshot.snapshotId, lastTime: { ticks: "1000", timescale: { magnitude: "1", unit: "ns" } } });
    expect(result.waveform).toMatchObject({ runId: result.runId, snapshotId: snapshot.snapshotId, complete: true, storage: "opfs" });
    expect(artifacts.put).toHaveBeenCalledWith(result.waveform, vcd);
    expect(stored).toHaveLength(1);
  });

  it("does not claim separate compile or run handles", async () => {
    const controller = new BrowserIdeController(createMockServices().workspace);
    await expect(controller.compile({
      projectId: "demo-project", snapshotId: "snapshot-1", topModule: "counter_tb", includeDirs: [], defines: {}, mode: "full",
    })).rejects.toMatchObject({ code: "UNSUPPORTED_CAPABILITY" });
    await expect(controller.run({
      projectId: "demo-project", snapshotId: "snapshot-1", compileId: "compile-1",
      until: { ticks: "100", timescale: { magnitude: "1", unit: "ns" } },
      workBudget: { maxSteps: 1, maxWallTimeMs: 1000, maxOutputBytes: 1024, maxVcdBytes: 1024 },
    })).rejects.toMatchObject({ code: "UNSUPPORTED_CAPABILITY" });
  });

  it("notifies the IDE when authenticated workspace operations change files", async () => {
    const services = createMockServices();
    const controller = new BrowserIdeController(services.workspace);
    const changed = vi.fn();
    controller.setOnWorkspaceChanged(changed);
    const path = ProjectPathSchema.parse("from-agent.sv");
    const created = await controller.writeFile(path, new TextEncoder().encode("module from_agent; endmodule"), null);
    expect(changed).toHaveBeenCalledTimes(1);
    await controller.deleteFile(path, created.revision);
    expect(changed).toHaveBeenCalledTimes(2);
  });

  it("rejects stale snapshots and missing WASM without invoking a native fallback", async () => {
    const services = createMockServices();
    const worker = vi.fn();
    const controller = new BrowserIdeController(services.workspace, { simulatorAvailable: false, runProjectWorker: worker });
    await expect(controller.execute(request({ snapshotId: "other-snapshot" }))).rejects.toMatchObject({ code: "SIMULATOR_UNAVAILABLE" });
    expect(worker).not.toHaveBeenCalled();

    const available = new BrowserIdeController(services.workspace, { simulatorAvailable: true, runProjectWorker: worker });
    await expect(available.execute(request({ snapshotId: "other-snapshot" }))).rejects.toMatchObject({ code: "REVISION_CONFLICT" });
    expect(worker).not.toHaveBeenCalled();
  });

  it("rejects fractional-nanosecond stops rather than rounding the simulation time", async () => {
    const services = createMockServices();
    const worker = vi.fn();
    const controller = new BrowserIdeController(services.workspace, { runProjectWorker: worker });
    const fractional = request({ until: { ticks: "1", timescale: { magnitude: "1", unit: "ps" } } });
    await expect(controller.execute(fractional)).rejects.toMatchObject({ code: "UNSUPPORTED_CAPABILITY" });
    expect(worker).not.toHaveBeenCalled();
  });

  it("reports Worker timeout errors explicitly", async () => {
    const services = createMockServices();
    const worker = vi.fn(async (): Promise<WorkerResponse> => ({ id: "worker-timeout", ok: false, error: { code: "TIMEOUT", message: "Xezim exceeded its wall-time budget." } }));
    const controller = new BrowserIdeController(stableWorkspace(services), { runProjectWorker: worker });
    const snapshot = await services.workspace.snapshot();
    await expect(controller.execute(request({ snapshotId: snapshot.snapshotId }))).rejects.toMatchObject<Partial<BrowserControllerError>>({
      code: "TIMEOUT",
      retryable: true,
    });
  });
});
