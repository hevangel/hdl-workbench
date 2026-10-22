import { afterEach, describe, expect, it, vi } from "vitest";
import { dispatchSurferCommand, SurferAdapterError, SurferIframeAdapter } from "../src/surfer/SurferIframeAdapter.js";
import type { WaveformArtifact } from "@xezim-ide/protocol";

const artifacts: Array<{ adapter: SurferIframeAdapter; iframe: HTMLIFrameElement }> = [];

function createAdapter(options: ConstructorParameters<typeof SurferIframeAdapter>[1] = {}, respondToProbe = true) {
  const iframe = document.createElement("iframe");
  document.body.append(iframe);
  const frameWindow = iframe.contentWindow;
  if (!frameWindow) throw new Error("jsdom did not create an iframe window");
  const messages: Array<{ data: unknown; targetOrigin: string }> = [];
  vi.spyOn(frameWindow, "postMessage").mockImplementation((data, targetOrigin) => {
    messages.push({ data, targetOrigin: typeof targetOrigin === "string" ? targetOrigin : targetOrigin?.targetOrigin ?? "" });
    if (respondToProbe && typeof data === "object" && data !== null && "type" in data && data.type === "xezim-surfer-ready-probe") {
      const probe = data as { nonce: string };
      queueMicrotask(() => window.dispatchEvent(new MessageEvent("message", {
        data: { type: "surfer-ready", nonce: probe.nonce },
        origin: window.location.origin,
        source: frameWindow,
      })));
    }
  });
  const adapter = new SurferIframeAdapter(iframe, options);
  artifacts.push({ adapter, iframe });
  return { adapter, iframe, messages };
}

const artifact: WaveformArtifact = {
  id: "wave-1",
  runId: "run-1",
  snapshotId: "snapshot-1",
  format: "vcd",
  sizeBytes: 441,
  storage: "bridge-file",
  timescale: { magnitude: "1", unit: "ns" },
  complete: true,
};

afterEach(() => {
  for (const { adapter, iframe } of artifacts.splice(0)) {
    adapter.dispose();
    iframe.remove();
  }
  vi.restoreAllMocks();
});

describe("SurferIframeAdapter", () => {
  it("uses the pinned wrapper messages for VCD load, signal add, cursor, and zoom", async () => {
    const { adapter, messages } = createAdapter({
      resolveWaveformUrl: () => `blob:${window.location.origin}/wave-1`,
    });

    await adapter.loadWaveform(artifact);
    await adapter.addSignals(["top.count"]);
    await adapter.setCursor({ ticks: "25" });
    await adapter.zoom({ ticks: "0" }, { ticks: "40" });

    const commands = messages.map(({ data }) => data).filter((data): data is Record<string, unknown> => typeof data === "object" && data !== null && "command" in data);
    expect(commands[0]).toEqual({ command: "LoadUrl", url: `blob:${window.location.origin}/wave-1` });
    expect(JSON.parse(String(commands[1]!.message))).toEqual({
      AddVariables: [{ path: { strs: ["top"], id: "None" }, name: "count", id: "None", index: null }],
    });
    expect(JSON.parse(String(commands[2]!.message))).toEqual({ CursorSet: [1, [25]] });
    expect(JSON.parse(String(commands[3]!.message))).toEqual({
      ZoomToRange: { start: [0, []], end: [1, [40]], viewport_idx: 0 },
    });
    expect(commands.every((message) => messages.find((item) => item.data === message)?.targetOrigin === window.location.origin)).toBe(true);
  });

  it("rejects incomplete artifacts, unsupported origins, invalid signal paths, and reversed zoom ranges", async () => {
    const { adapter } = createAdapter();
    const incomplete = { ...artifact, complete: false };

    await expect(adapter.loadWaveform(incomplete)).rejects.toMatchObject({ code: "INCOMPLETE_ARTIFACT" });
    await expect(adapter.loadUrl("https://example.invalid/wave.vcd")).rejects.toMatchObject({ code: "CROSS_ORIGIN_URL" });
    await expect(adapter.addSignals(["top..count"])).rejects.toMatchObject({ code: "INVALID_SIGNAL_PATH" });
    await expect(adapter.zoom({ ticks: "10" }, { ticks: "10" })).rejects.toMatchObject({ code: "INVALID_TIME_RANGE" });
    await expect(dispatchSurferCommand(adapter, { op: "deleteAllWaveforms" })).rejects.toMatchObject({ code: "UNKNOWN_COMMAND" });
  });

  it("requires an artifact resolver instead of guessing an artifact URL", async () => {
    const { adapter } = createAdapter();
    await expect(adapter.loadWaveform(artifact)).rejects.toMatchObject({ code: "ARTIFACT_RESOLVER_UNAVAILABLE" });
  });

  it("repeats the readiness handshake after the iframe reloads", async () => {
    const { adapter, iframe, messages } = createAdapter();
    await adapter.ready();
    const firstProbeCount = messages.filter(({ data }) => typeof data === "object" && data !== null && "type" in data && data.type === "xezim-surfer-ready-probe").length;

    iframe.dispatchEvent(new Event("load"));
    await adapter.ready();

    const secondProbeCount = messages.filter(({ data }) => typeof data === "object" && data !== null && "type" in data && data.type === "xezim-surfer-ready-probe").length;
    expect(secondProbeCount).toBeGreaterThan(firstProbeCount);
  });

  it("ignores readiness messages from the wrong origin or frame", async () => {
    const { adapter, iframe } = createAdapter({ readyTimeoutMs: 20 }, false);
    const readiness = adapter.ready();
    const frameWindow = iframe.contentWindow!;
    const probe = vi.mocked(frameWindow.postMessage).mock.calls[0]?.[0] as { nonce: string };
    window.dispatchEvent(new MessageEvent("message", {
      data: { type: "surfer-ready", nonce: probe.nonce },
      origin: "https://attacker.invalid",
      source: frameWindow,
    }));
    window.dispatchEvent(new MessageEvent("message", {
      data: { type: "surfer-ready", nonce: probe.nonce },
      origin: window.location.origin,
      source: window,
    }));

    await expect(readiness).rejects.toBeInstanceOf(SurferAdapterError);
    await expect(readiness).rejects.toMatchObject({ code: "READY_TIMEOUT" });
  });
});
