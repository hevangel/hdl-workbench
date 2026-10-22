import { describe, expect, it, vi } from "vitest";
import type { RunResult } from "@xezim-ide/protocol";
import { BrowserIdeController } from "../src/controller/BrowserIdeController.js";
import { McpBrowserClient } from "../src/controller/McpBrowserClient.js";
import { createMockServices } from "../src/mockServices.js";

class FakeSocket extends EventTarget {
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;
  readyState = 0;
  sent: string[] = [];
  constructor(readonly url: string) {
    super();
    queueMicrotask(() => {
      this.readyState = FakeSocket.OPEN;
      this.dispatchEvent(new Event("open"));
    });
  }
  send(value: string): void {
    this.sent.push(value);
    const parsed = JSON.parse(value) as { type?: string; challengeId?: string };
    if (parsed.type === "pair") {
      queueMicrotask(() => this.receive({
        type: "paired",
        challengeId: parsed.challengeId,
        sessionToken: "f".repeat(64),
        role: "write",
        expiresAt: new Date(Date.now() + 60_000).toISOString(),
      }));
    }
  }
  close(code = 1000, reason = "closed"): void {
    this.readyState = FakeSocket.CLOSED;
    this.dispatchEvent(new CloseEvent("close", { code, reason }));
  }
  receive(value: unknown): void {
    this.dispatchEvent(new MessageEvent("message", { data: JSON.stringify(value) }));
  }
}

const fakeSocketClass = FakeSocket as unknown as typeof WebSocket;

describe("McpBrowserClient", () => {
  it("pairs with a local challenge and routes the same one-shot controller operation as the Run button", async () => {
    const services = createMockServices();
    const workspace = {
      ...services.workspace,
      snapshot: async () => ({ ...(await services.workspace.snapshot()), snapshotId: "mock-snapshot-1" }),
    };
    const controller = new BrowserIdeController(workspace);
    const result: RunResult = {
      runId: "run-mcp-1",
      projectId: "local-project",
      snapshotId: "snap-0123456789abcdef0123456789abcdef",
      status: "finish",
      diagnostics: [],
      stdout: "COUNT=4\n",
      stderr: "",
      lastTime: { ticks: "1000", timescale: { magnitude: "1", unit: "ns" } },
    };
    const execute = vi.spyOn(controller, "execute").mockResolvedValue(result);
    const client = new McpBrowserClient(controller, { WebSocketClass: fakeSocketClass });
    const challenge = {
      challengeId: "challenge-1",
      code: "ABCD2345",
      expiresAt: new Date(Date.now() + 60_000).toISOString(),
    };

    await client.connect({ websocketUrl: "ws://127.0.0.1:47531/ws", challenge });
    expect(client.currentState).toBe("connected");
    const socket = (client as unknown as { socket: FakeSocket }).socket;
    socket.receive({
      type: "rpc.request",
      token: "f".repeat(64),
      request: {
        protocol: "xezim-ide/1",
        id: "execute-1",
        op: "simulation.execute",
        projectId: "local-project",
        params: {
          projectId: "local-project",
          snapshotId: result.snapshotId,
          topModule: "counter_tb",
          until: { ticks: "1000", timescale: { magnitude: "1", unit: "ns" } },
          workBudget: { maxWallTimeMs: 5000, maxOutputBytes: 1_048_576, maxVcdBytes: 64 * 1024 * 1024 },
        },
      },
    });
    await vi.waitFor(() => expect(socket.sent.some((message) => JSON.parse(message).type === "rpc.response")).toBe(true));

    expect(execute).toHaveBeenCalledWith(expect.objectContaining({
      projectId: "local-project",
      snapshotId: result.snapshotId,
      topModule: "counter_tb",
    }));
    const response = JSON.parse(socket.sent.find((message) => JSON.parse(message).type === "rpc.response")!) as {
      response: { id: string; ok: boolean; result: RunResult };
    };
    expect(response.response).toMatchObject({ id: "execute-1", ok: true, result: { runId: "run-mcp-1", stdout: "COUNT=4\n" } });
    client.disconnect();
    expect(client.currentState).toBe("disconnected");
  });

  it("rejects non-loopback bridge URLs before opening a socket", async () => {
    const controller = new BrowserIdeController(createMockServices().workspace);
    const socketConstructor = vi.fn();
    const client = new McpBrowserClient(controller, { WebSocketClass: socketConstructor as unknown as typeof WebSocket });
    await expect(client.connect({
      websocketUrl: "ws://attacker.example/ws",
      challenge: { challengeId: "challenge-1", code: "ABCD2345", expiresAt: new Date(Date.now() + 60_000).toISOString() },
    })).rejects.toThrow("must use ws://127.0.0.1");
    expect(socketConstructor).not.toHaveBeenCalled();
  });
});
