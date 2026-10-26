import { beforeEach, describe, expect, it, vi } from "vitest";
import { ProjectPathSchema, type ExecuteRequest } from "@xezim-ide/protocol";
import { BrowserControllerError, BrowserIdeController } from "../src/controller/BrowserIdeController.js";
import { createMockServices } from "../src/mockServices.js";
import type { DpiCompilerResponse } from "../src/workers/dpiCompilerProtocol.js";
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

  it("completes a run whose design never dumps a waveform", async () => {
    const services = createMockServices();
    const artifacts = { put: vi.fn(), get: vi.fn(), objectUrl: vi.fn(), dispose: vi.fn() } as unknown as OpfsWaveformStore;
    const worker = vi.fn(async (input): Promise<WorkerResponse> => ({
      id: "worker-1",
      ok: true,
      result: {
        schemaVersion: 1, status: "completed", maxTimeNs: "1000", lastTimeTicks: "1000", finished: true, sawFatal: false,
        stdout: "hello\n", diagnostics: [], snapshotId: input.snapshotId, fileOutputs: [],
      },
    }));
    const workspace = stableWorkspace(services);
    const controller = new BrowserIdeController(workspace, { runProjectWorker: worker, artifactStore: artifacts });
    const result = await controller.execute(request({ snapshotId: (await workspace.snapshot()).snapshotId }));

    expect(result).toMatchObject({ status: "finish", stdout: "hello\n", lastTime: { ticks: "1000", timescale: { magnitude: "1", unit: "ns" } } });
    expect(result.waveform).toBeUndefined();
    expect(artifacts.put).not.toHaveBeenCalled();
  });

  it("passes plusargs to the Worker and stores an FST result as an FST artifact", async () => {
    const services = createMockServices();
    const put = vi.fn();
    const artifacts = { put, get: vi.fn(), objectUrl: vi.fn(), dispose: vi.fn() } as unknown as OpfsWaveformStore;
    const fstBytes = Uint8Array.from([0, 1, 2, 250, 251, 252, 253]);
    const worker = vi.fn(async (input): Promise<WorkerResponse> => ({
      id: "worker-1",
      ok: true,
      result: {
        schemaVersion: 1, status: "completed", maxTimeNs: "1000", lastTimeTicks: "1000", finished: true, sawFatal: false,
        stdout: "", diagnostics: [], snapshotId: input.snapshotId, fileOutputs: [],
        waveform: { format: "fst", sizeBytes: fstBytes.byteLength, complete: true, timescale: "1 ns" },
        fstBase64: btoa(String.fromCharCode(...fstBytes)),
      },
    }));
    const workspace = stableWorkspace(services);
    const controller = new BrowserIdeController(workspace, { runProjectWorker: worker, artifactStore: artifacts });
    const result = await controller.execute(request({
      snapshotId: (await workspace.snapshot()).snapshotId,
      plusargs: ["+verbose", "+N=42"],
      waveformFormat: "fst",
    }));

    expect(worker).toHaveBeenCalledWith(expect.objectContaining({ plusargs: ["+verbose", "+N=42"], waveformFormat: "fst" }), expect.anything());
    expect(result.waveform).toMatchObject({ format: "fst", sizeBytes: fstBytes.byteLength, complete: true });
    expect(put).toHaveBeenCalledWith(result.waveform, fstBytes);
  });

  it("omits plusargs and format from the Worker request when none were asked for", async () => {
    const services = createMockServices();
    const artifacts = { put: vi.fn(), get: vi.fn(), objectUrl: vi.fn(), dispose: vi.fn() } as unknown as OpfsWaveformStore;
    const worker = vi.fn(async (input): Promise<WorkerResponse> => ({
      id: "worker-1",
      ok: true,
      result: { schemaVersion: 1, status: "completed", maxTimeNs: "1000", lastTimeTicks: "1000", finished: true, sawFatal: false, stdout: "", diagnostics: [], snapshotId: input.snapshotId },
    }));
    const workspace = stableWorkspace(services);
    const controller = new BrowserIdeController(workspace, { runProjectWorker: worker, artifactStore: artifacts });
    await controller.execute(request({ snapshotId: (await workspace.snapshot()).snapshotId }));
    expect(worker.mock.calls[0]?.[0]).not.toHaveProperty("plusargs");
    expect(worker.mock.calls[0]?.[0]).not.toHaveProperty("waveformFormat");
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

  it("enforces Xezim's project-input byte limit before posting data to the Worker", async () => {
    const services = createMockServices();
    const largePath = ProjectPathSchema.parse("large.sv");
    await services.workspace.writeFile(largePath, new Uint8Array(16 * 1024 * 1024), null);
    const workspace = stableWorkspace(services);
    const worker = vi.fn();
    const controller = new BrowserIdeController(workspace, { runProjectWorker: worker });
    const snapshot = await workspace.snapshot();

    await expect(controller.execute(request({ snapshotId: snapshot.snapshotId }))).rejects.toMatchObject({ code: "RESOURCE_LIMIT" });
    expect(worker).not.toHaveBeenCalled();
  });

  it("routes precompiled Wasm DPI modules separately from UTF-8 project files", async () => {
    const services = createMockServices();
    const moduleBytes = Uint8Array.from([0, 97, 115, 109, 1, 0, 0, 0]);
    await services.workspace.writeFile(ProjectPathSchema.parse("dpi/add.wasm"), moduleBytes, null);
    const workspace = stableWorkspace(services);
    const worker = vi.fn(async (input): Promise<WorkerResponse> => ({
      id: "worker-dpi",
      ok: true,
      result: {
        schemaVersion: 1,
        status: "completed",
        maxTimeNs: "1000",
        lastTimeTicks: "1000",
        finished: false,
        sawFatal: false,
        stdout: "",
        diagnostics: [],
        snapshotId: input.snapshotId,
        waveform: { format: "vcd", sizeBytes: 0, complete: false, timescale: "1 ns" },
      },
    }));
    const controller = new BrowserIdeController(workspace, { runProjectWorker: worker });
    await controller.execute(request());

    const posted = worker.mock.calls[0]?.[0];
    expect(posted).toBeDefined();
    if (!posted) throw new Error("The DPI project was not sent to the Worker.");
    expect(posted.files).toEqual(expect.arrayContaining([expect.objectContaining({ path: "rtl/counter.sv" })]));
    expect(posted.dpiModules).toHaveLength(1);
    expect(posted.dpiModules?.[0]?.path).toBe("dpi/add.wasm");
    expect([...new Uint8Array(posted.dpiModules?.[0]?.bytes ?? new ArrayBuffer(0))]).toEqual([...moduleBytes]);
  });

  it("compiles C++ sources in a separate Worker and loads the resulting Wasm module", async () => {
    const services = createMockServices();
    await services.workspace.writeFile(ProjectPathSchema.parse("dpi/reference.cpp"), new TextEncoder().encode("extern \"C\" int add(int a, int b) { return a + b; }"), null);
    const workspace = stableWorkspace(services);
    const compiledBytes = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]).buffer;
    const compileWorker = vi.fn(async (files): Promise<DpiCompilerResponse> => ({
      id: "compiler-1",
      ok: true,
      modules: [{ path: "dpi/reference.cpp", bytes: compiledBytes }],
      output: "",
    }));
    const worker = vi.fn(async (input): Promise<WorkerResponse> => ({
      id: "worker-dpi",
      ok: true,
      result: {
        schemaVersion: 1,
        status: "completed",
        maxTimeNs: "1000",
        lastTimeTicks: "1000",
        finished: false,
        sawFatal: false,
        stdout: "",
        diagnostics: [],
        snapshotId: input.snapshotId,
        waveform: { format: "vcd", sizeBytes: 0, complete: false, timescale: "1 ns" },
      },
    }));
    const controller = new BrowserIdeController(workspace, {
      runProjectWorker: worker,
      compileDpiProjectWorker: compileWorker,
    });

    await controller.execute(request());
    expect(compileWorker).toHaveBeenCalledWith(
      expect.arrayContaining([expect.objectContaining({ path: "dpi/reference.cpp" })]),
      expect.objectContaining({ timeoutMs: expect.any(Number) }),
    );
    expect(worker.mock.calls[0]?.[0].dpiModules?.[0]).toEqual({ path: "dpi/reference.cpp.compiled.wasm", bytes: compiledBytes });
  });

  it("reports C/C++ compilation errors without starting Xezim", async () => {
    const services = createMockServices();
    await services.workspace.writeFile(ProjectPathSchema.parse("dpi/reference.c"), new TextEncoder().encode("int add(int a, int b) { return a + b; }"), null);
    const workspace = stableWorkspace(services);
    const compileWorker = vi.fn(async (): Promise<DpiCompilerResponse> => ({
      id: "compiler-1",
      ok: false,
      error: { code: "DPI_COMPILATION_FAILED", message: "undefined symbol in C source" },
    }));
    const worker = vi.fn();
    const controller = new BrowserIdeController(workspace, {
      runProjectWorker: worker,
      compileDpiProjectWorker: compileWorker,
    });

    await expect(controller.execute(request())).rejects.toMatchObject({
      code: "DPI_COMPILATION_FAILED",
      message: "undefined symbol in C source",
    });
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
