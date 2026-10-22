import { afterEach, describe, expect, it } from "vitest";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { promises as fs } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { WebSocket } from "ws";
import { PROTOCOL_VERSION, type RpcRequest, type RpcResponse } from "@xezim-ide/protocol";
import { WebSocketBridge } from "../src/websocketBridge.js";

const ORIGIN = "http://localhost:5173";
const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const serverEntry = path.join(packageRoot, "dist", "main.js");
const inspectorEntry = path.join(packageRoot, "node_modules", "@modelcontextprotocol", "inspector", "clients", "launcher", "build", "index.js");

interface TestContext {
  root: string;
  client: Client;
  transport: StdioClientTransport;
  stderr: string;
}

const active: TestContext[] = [];

function parseText(result: { content: Array<{ type: string; text?: string }> }): unknown {
  const block = result.content.find((item) => item.type === "text");
  if (!block?.text) throw new Error("MCP tool returned no text content");
  return JSON.parse(block.text) as unknown;
}

function revision(bytes: Uint8Array): string {
  return `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

async function startBridge(requestTimeoutMs = 1_000): Promise<TestContext> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "xezim-mcp-"));
  await fs.writeFile(path.join(root, "top.sv"), "module top; endmodule\n");
  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [serverEntry, "--project-root", root, "--port", "0", "--allowed-origins", ORIGIN, "--request-timeout-ms", String(requestTimeoutMs)],
    cwd: packageRoot,
    stderr: "pipe",
  });
  let stderr = "";
  transport.stderr?.on("data", (chunk: Buffer) => { stderr += chunk.toString("utf8"); });
  const client = new Client({ name: "xezim-mcp-acceptance", version: "0.1.0" });
  await client.connect(transport);
  const context = { root, client, transport, get stderr() { return stderr; } } as TestContext;
  active.push(context);
  return context;
}

async function waitForOpen(socket: WebSocket): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    socket.once("open", resolve);
    socket.once("error", reject);
  });
}

async function nextSocketMessage(socket: WebSocket): Promise<Record<string, unknown>> {
  return await new Promise((resolve, reject) => {
    socket.once("message", (data) => {
      try {
        resolve(JSON.parse(data.toString()) as Record<string, unknown>);
      } catch (error) {
        reject(error);
      }
    });
    socket.once("error", reject);
  });
}

class MockBrowser {
  readonly socket: WebSocket;
  readonly files = new Map<string, Uint8Array>();
  readonly runs = new Map<string, unknown>();
  readonly runEntered = deferred<void>();
  holdRun = false;
  ignoredOperation: string | null = null;
  compileCount = 0;
  runCount = 0;

  private constructor(socket: WebSocket, pairing: Record<string, unknown>) {
    this.socket = socket;
    this.socket.on("message", (raw) => {
      const envelope = JSON.parse(raw.toString()) as { type: string; token: string; request: RpcRequest };
      if (envelope.type !== "rpc.request") return;
      void this.onRequest(envelope.request, envelope.token);
    });
    if (typeof pairing.sessionToken !== "string") throw new Error("Pairing did not return a session token");
  }

  static async connect(status: Record<string, unknown>): Promise<MockBrowser> {
    const url = status.websocketUrl;
    const pairing = status.pairingChallenge as { challengeId: string; code: string };
    if (typeof url !== "string") throw new Error("Missing WebSocket URL");
    const socket = new WebSocket(url, { origin: ORIGIN });
    await waitForOpen(socket);
    socket.send(JSON.stringify({ type: "pair", challengeId: pairing.challengeId, code: pairing.code, clientName: "mock-browser", requestedRole: "write" }));
    const paired = await nextSocketMessage(socket);
    return new MockBrowser(socket, paired);
  }

  async onRequest(request: RpcRequest, token: string): Promise<void> {
    if (this.ignoredOperation === request.op) return;
    let result: unknown;
    switch (request.op) {
      case "connection.get_status":
        result = {
          protocol: PROTOCOL_VERSION,
          connected: true,
          capabilities: [
            { name: "browser_simulation", available: true },
            { name: "waveform_load", available: true },
            { name: "waveform_add_signals", available: true },
            { name: "waveform_cursor", available: true },
          ],
        };
        break;
      case "workspace.list_files":
        result = { files: [...this.files].map(([filePath, bytes]) => ({ path: filePath, revision: revision(bytes), sizeBytes: bytes.byteLength })) };
        break;
      case "workspace.write_file": {
        const previous = this.files.get(request.params.path);
        const previousRevision = previous ? revision(previous) : null;
        if (previousRevision !== request.params.expectedRevision) {
          this.send(request, token, { ok: false, error: { code: "REVISION_CONFLICT", message: "Browser file changed" } });
          return;
        }
        const bytes = Buffer.from(request.params.contentBase64, "base64");
        this.files.set(request.params.path, bytes);
        result = { revision: revision(bytes) };
        break;
      }
      case "workspace.delete_file": {
        const previous = this.files.get(request.params.path);
        if (!previous || revision(previous) !== request.params.expectedRevision) {
          this.send(request, token, { ok: false, error: { code: "REVISION_CONFLICT", message: "Browser file changed" } });
          return;
        }
        this.files.delete(request.params.path);
        result = { deleted: true };
        break;
      }
      case "simulation.compile": {
        this.compileCount += 1;
        if (!this.files.has("top.sv")) throw new Error("Snapshot was not synchronized before compile");
        result = { compileId: `compile-${this.compileCount}`, projectId: request.projectId, snapshotId: request.params.snapshotId, success: true, diagnostics: [] };
        break;
      }
      case "simulation.run":
      case "simulation.execute": {
        this.runCount += 1;
        this.runEntered.resolve();
        if (this.holdRun) return;
        const resultValue = {
          runId: `run-${this.runCount}`,
          projectId: request.projectId,
          snapshotId: request.params.snapshotId,
          status: "completed",
          diagnostics: [],
          stdout: "simulation complete\n",
          stderr: "",
          lastTime: { ticks: request.params.until.ticks, timescale: request.params.until.timescale },
        };
        this.runs.set(resultValue.runId, resultValue);
        result = resultValue;
        break;
      }
      case "simulation.get_diagnostics":
        result = { diagnostics: [] };
        break;
      case "simulation.list_runs":
        result = { runs: [...this.runs.values()].slice(0, request.params.limit) };
        break;
      case "simulation.get_log":
        result = { runId: request.params.runId, stdout: "simulation complete\n", stderr: "" };
        break;
      case "waveform.show":
        result = { artifactId: request.params.artifactId, opened: true };
        break;
    }
    this.send(request, token, { ok: true, result });
  }

  private send(request: RpcRequest, token: string, response: Omit<RpcResponse, "protocol" | "id">): void {
    this.socket.send(JSON.stringify({ type: "rpc.response", token, response: { protocol: PROTOCOL_VERSION, id: request.id, ...response } }));
  }
}

async function errorCode(result: Awaited<ReturnType<Client["callTool"]>>): Promise<string> {
  const parsed = parseText(result) as { code?: string };
  return parsed.code ?? "";
}

afterEach(async () => {
  for (const context of active.splice(0)) {
    await context.client.close().catch(() => undefined);
    await fs.rm(context.root, { recursive: true, force: true });
  }
});

describe("LUNA-07 MCP bridge integration", () => {
  it("lets the pinned official MCP Inspector CLI call get_connection_status", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "xezim-inspector-"));
    await fs.writeFile(path.join(root, "top.sv"), "module top; endmodule\n");
    try {
      const result = spawnSync(process.execPath, [
        inspectorEntry, "--cli", process.execPath, serverEntry,
        "--project-root", root,
        "--port", "0",
        "--allowed-origins", ORIGIN,
        "--method", "tools/call",
        "--tool-name", "get_connection_status",
      ], { cwd: packageRoot, encoding: "utf8", timeout: 15_000, windowsHide: true });
      expect(result.error).toBeUndefined();
      expect(result.status).toBe(0);
      expect(result.stdout).toContain("pairingChallenge");
      expect(result.stdout).toContain("websocketUrl");
      expect(result.stderr).toContain("MCP stdio ready");
    } finally {
      await fs.rm(root, { recursive: true, force: true });
    }
  });

  it("serves all typed tools over MCP stdio and forwards snapshots and runs to the browser", async () => {
    const context = await startBridge();
    const { tools } = await context.client.listTools();
    expect(tools.map((item) => item.name).sort()).toEqual([
      "compile", "get_connection_status", "get_diagnostics", "get_simulation_log", "list_project_files", "list_runs",
      "read_project_file", "run_simulation", "show_waveforms", "write_project_file",
    ].sort());

    const status = parseText(await context.client.callTool({ name: "get_connection_status", arguments: {} })) as Record<string, unknown>;
    expect(status.connected).toBe(false);
    expect(String(status.websocketUrl)).toMatch(/^ws:\/\/127\.0\.0\.1:\d+\/ws$/);
    const browser = await MockBrowser.connect(status);

    const listed = parseText(await context.client.callTool({ name: "list_project_files", arguments: {} })) as { files: Array<{ path: string; revision: string }> };
    expect(listed.files.map((item) => item.path)).toEqual(["top.sv"]);
    const read = parseText(await context.client.callTool({ name: "read_project_file", arguments: { path: "top.sv" } })) as { revision: string; contentBase64: string };
    expect(Buffer.from(read.contentBase64, "base64").toString("utf8")).toContain("module top");

    const firstWrite = await context.client.callTool({
      name: "write_project_file",
      arguments: { path: "top.sv", expected_revision: read.revision, content_base64: Buffer.from("module top; logic ready; endmodule\n").toString("base64") },
    });
    const updated = parseText(firstWrite) as { revision: string };
    expect(updated.revision).not.toBe(read.revision);
    const conflict = await context.client.callTool({
      name: "write_project_file",
      arguments: { path: "top.sv", expected_revision: read.revision, content_base64: Buffer.from("module stale; endmodule").toString("base64") },
    });
    expect(conflict.isError).toBe(true);
    expect(await errorCode(conflict)).toBe("REVISION_CONFLICT");

    const compile = parseText(await context.client.callTool({ name: "compile", arguments: { top_module: "top" } })) as { compileId: string; success: boolean };
    expect(compile).toMatchObject({ compileId: "compile-1", success: true });
    expect(Buffer.from(browser.files.get("top.sv") ?? []).toString("utf8")).toContain("logic ready");
    const run = parseText(await context.client.callTool({
      name: "run_simulation",
      arguments: { top_module: "top", until_ticks: "100", timescale: { magnitude: "1", unit: "ns" } },
    })) as { runId: string; status: string };
    expect(run).toMatchObject({ runId: "run-1", status: "completed" });
    expect(parseText(await context.client.callTool({ name: "get_diagnostics", arguments: { run_id: run.runId } }))).toEqual({ diagnostics: [] });
    expect(parseText(await context.client.callTool({ name: "list_runs", arguments: { limit: 5 } }))).toMatchObject({ runs: [{ runId: run.runId }] });
    expect(parseText(await context.client.callTool({ name: "show_waveforms", arguments: { artifact_id: "artifact-1", cursor: { ticks: "25" } } }))).toEqual({ artifactId: "artifact-1", opened: true });
    expect(parseText(await context.client.callTool({ name: "get_simulation_log", arguments: { run_id: run.runId } }))).toMatchObject({ stdout: "simulation complete\n" });
    expect(context.stderr).toContain("MCP stdio ready");
    browser.socket.close();
  });

  it("rejects untrusted origins, missing or invalid tokens, traversal, and symlink escapes", async () => {
    const context = await startBridge();
    const status = parseText(await context.client.callTool({ name: "get_connection_status", arguments: {} })) as Record<string, unknown>;
    const rejected = new WebSocket(String(status.websocketUrl), { origin: "https://evil.example" });
    const statusCode = await new Promise<number>((resolve, reject) => {
      rejected.once("unexpected-response", (_request, response) => resolve(response.statusCode ?? 0));
      rejected.once("error", (error) => reject(error));
    });
    expect(statusCode).toBe(403);

    const missingToken = new WebSocket(String(status.websocketUrl), { origin: ORIGIN });
    await waitForOpen(missingToken);
    missingToken.send(JSON.stringify({ type: "rpc.response", response: {} }));
    const missingTokenClose = await new Promise<number>((resolve) => missingToken.once("close", (code) => resolve(code)));
    expect(missingTokenClose).toBe(4401);

    const traversal = await context.client.callTool({ name: "read_project_file", arguments: { path: "../secret.txt" } });
    expect(traversal.isError).toBe(true);
    const absolute = await context.client.callTool({ name: "read_project_file", arguments: { path: path.join(context.root, "top.sv") } });
    expect(absolute.isError).toBe(true);
    const alternateStream = await context.client.callTool({ name: "read_project_file", arguments: { path: "top.sv:metadata" } });
    expect(alternateStream.isError).toBe(true);

    const outside = await fs.mkdtemp(path.join(os.tmpdir(), "xezim-outside-"));
    try {
      await fs.writeFile(path.join(outside, "secret.txt"), "outside");
      await fs.symlink(outside, path.join(context.root, "escape"), "junction");
      const escaped = await context.client.callTool({ name: "read_project_file", arguments: { path: "escape/secret.txt" } });
      expect(escaped.isError).toBe(true);
      expect(await errorCode(escaped)).toBe("INVALID_PATH");
    } finally {
      await fs.rm(outside, { recursive: true, force: true });
    }

    const browser = await MockBrowser.connect(status);
    browser.socket.send(JSON.stringify({ type: "rpc.response", token: "wrong-token", response: { protocol: PROTOCOL_VERSION, id: "forged", ok: true, result: {} } }));
    const closeCode = await new Promise<number>((resolve) => browser.socket.once("close", (code) => resolve(code)));
    expect(closeCode).toBe(4401);
  });

  it("returns a deterministic busy result while simulation work is pending", async () => {
    const context = await startBridge();
    const status = parseText(await context.client.callTool({ name: "get_connection_status", arguments: {} })) as Record<string, unknown>;
    const browser = await MockBrowser.connect(status);
    browser.holdRun = true;
    const runArguments = { top_module: "top", until_ticks: "10", timescale: { magnitude: "1", unit: "ns" } };
    const first = context.client.callTool({ name: "run_simulation", arguments: runArguments });
    await browser.runEntered.promise;
    const second = await context.client.callTool({ name: "run_simulation", arguments: runArguments });
    expect(second.isError).toBe(true);
    expect(await errorCode(second)).toBe("SIMULATION_BUSY");
    browser.socket.close();
    const firstResult = await first;
    expect(firstResult.isError).toBe(true);
    expect(await errorCode(firstResult)).toBe("BROWSER_DISCONNECTED");
  });

  it("reports browser disconnects and request timeouts without native fallback", async () => {
    const disconnectedContext = await startBridge(300);
    const disconnectedStatus = parseText(await disconnectedContext.client.callTool({ name: "get_connection_status", arguments: {} })) as Record<string, unknown>;
    const browser = await MockBrowser.connect(disconnectedStatus);
    browser.socket.close();
    await new Promise<void>((resolve) => setTimeout(resolve, 25));
    const disconnected = await disconnectedContext.client.callTool({ name: "compile", arguments: { top_module: "top" } });
    expect(disconnected.isError).toBe(true);
    expect(await errorCode(disconnected)).toBe("BROWSER_DISCONNECTED");

    const timeoutContext = await startBridge(100);
    const timeoutStatus = parseText(await timeoutContext.client.callTool({ name: "get_connection_status", arguments: {} })) as Record<string, unknown>;
    const timeoutBrowser = await MockBrowser.connect(timeoutStatus);
    timeoutBrowser.ignoredOperation = "simulation.compile";
    const timeout = await timeoutContext.client.callTool({ name: "compile", arguments: { top_module: "top" } });
    expect(timeout.isError).toBe(true);
    expect(await errorCode(timeout)).toBe("TIMEOUT");
    timeoutBrowser.socket.close();
  });

  it("expires a pairing challenge and rejects its stale code", async () => {
    const bridge = new WebSocketBridge({ allowedOrigins: [ORIGIN], port: 0, pairingTtlMs: 20 });
    await bridge.start();
    try {
      const stale = bridge.status().pairingChallenge;
      expect(stale).not.toBeNull();
      await new Promise<void>((resolve) => setTimeout(resolve, 30));
      const socket = new WebSocket(bridge.url, { origin: ORIGIN });
      await waitForOpen(socket);
      socket.send(JSON.stringify({ type: "pair", challengeId: stale?.challengeId, code: stale?.code, clientName: "expired-mock", requestedRole: "write" }));
      const closeCode = await new Promise<number>((resolve) => socket.once("close", (code) => resolve(code)));
      expect(closeCode).toBe(4401);
    } finally {
      await bridge.close();
    }
  });
});
