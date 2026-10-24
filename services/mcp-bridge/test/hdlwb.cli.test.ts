import { afterEach, describe, expect, it } from "vitest";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { promises as fs } from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { WebSocket } from "ws";
import { PROTOCOL_VERSION, type RpcRequest } from "@xezim-ide/protocol";

const ORIGIN = "http://localhost:5173";
const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cliEntry = path.join(packageRoot, "scripts", "hdlwb.mjs");

interface CliResult {
  code: number | null;
  stdout: string;
  stderr: string;
}

interface CliContext {
  root: string;
  stateDir: string;
  sockets: WebSocket[];
}

const active: CliContext[] = [];

function revision(bytes: Uint8Array): string {
  return `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
}

async function cli(context: CliContext, args: string[]): Promise<CliResult> {
  return await new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [cliEntry, ...args], {
      cwd: packageRoot,
      env: { ...process.env, HDLWB_STATE_DIR: context.stateDir, HDLWB_SESSION: "test" },
      windowsHide: true,
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk: Buffer) => { stdout += chunk.toString("utf8"); });
    child.stderr.on("data", (chunk: Buffer) => { stderr += chunk.toString("utf8"); });
    child.once("error", reject);
    child.once("close", (code) => resolve({ code, stdout, stderr }));
  });
}

async function createContext(): Promise<CliContext> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "hdlwb-project-"));
  const stateDir = await fs.mkdtemp(path.join(os.tmpdir(), "hdlwb-state-"));
  await fs.writeFile(path.join(root, "top.sv"), "module top; endmodule\n");
  const context = { root, stateDir, sockets: [] };
  active.push(context);
  return context;
}

// A minimal paired browser: it mirrors synchronized files and answers execute.
async function pairMockBrowser(context: CliContext, status: { websocketUrl: string; pairingChallenge: { challengeId: string; code: string } }): Promise<void> {
  const socket = new WebSocket(status.websocketUrl, { origin: ORIGIN });
  context.sockets.push(socket);
  await new Promise<void>((resolve, reject) => {
    socket.once("open", resolve);
    socket.once("error", reject);
  });
  const files = new Map<string, Uint8Array>();
  const paired = new Promise<void>((resolve) => socket.once("message", () => resolve()));
  socket.send(JSON.stringify({ type: "pair", challengeId: status.pairingChallenge.challengeId, code: status.pairingChallenge.code, clientName: "mock-browser", requestedRole: "write" }));
  await paired;
  socket.on("message", (raw) => {
    const envelope = JSON.parse(raw.toString()) as { type: string; token: string; request: RpcRequest };
    if (envelope.type !== "rpc.request") return;
    const { request, token } = envelope;
    let result: unknown;
    switch (request.op) {
      case "connection.get_status":
        result = { protocol: PROTOCOL_VERSION, connected: true, capabilities: [{ name: "browser_simulation", available: true }] };
        break;
      case "workspace.list_files":
        result = { files: [...files].map(([filePath, bytes]) => ({ path: filePath, revision: revision(bytes), sizeBytes: bytes.byteLength })) };
        break;
      case "workspace.write_file": {
        const bytes = Buffer.from(request.params.contentBase64, "base64");
        files.set(request.params.path, bytes);
        result = { revision: revision(bytes) };
        break;
      }
      case "workspace.delete_file":
        files.delete(request.params.path);
        result = { deleted: true };
        break;
      case "simulation.execute":
        result = {
          runId: "run-1",
          projectId: request.projectId,
          snapshotId: request.params.snapshotId,
          status: "finish",
          diagnostics: [],
          stdout: `files=${[...files.keys()].join(",")}\n`,
          stderr: "",
          lastTime: request.params.until,
        };
        break;
    }
    socket.send(JSON.stringify({ type: "rpc.response", token, response: { protocol: PROTOCOL_VERSION, id: request.id, ok: true, result } }));
  });
}

async function rawRequest(port: number, headers: Record<string, string>): Promise<number> {
  return await new Promise((resolve, reject) => {
    const req = http.request({ host: "127.0.0.1", port, method: "GET", path: "/health", headers }, (res) => {
      res.resume();
      resolve(res.statusCode ?? 0);
    });
    req.once("error", reject);
    req.end();
  });
}

afterEach(async () => {
  for (const context of active.splice(0)) {
    for (const socket of context.sockets) socket.close();
    await cli(context, ["stop"]);
    await fs.rm(context.root, { recursive: true, force: true });
    await fs.rm(context.stateDir, { recursive: true, force: true });
  }
});

describe("hdlwb session CLI", () => {
  it("keeps one paired bridge across separate commands", async () => {
    const context = await createContext();
    const started = await cli(context, ["start", "--project-root", context.root, "--port", "0", "--allowed-origins", ORIGIN]);
    expect(started.code, started.stderr).toBe(0);
    const startInfo = JSON.parse(started.stdout) as { connection: { connected: boolean; websocketUrl: string; pairingChallenge: { challengeId: string; code: string } } };
    expect(startInfo.connection.connected).toBe(false);

    const second = await cli(context, ["start", "--project-root", context.root]);
    expect(second.code).toBe(2);
    expect(second.stderr).toContain("already running");

    const tools = await cli(context, ["tools"]);
    expect((JSON.parse(tools.stdout) as Array<{ name: string }>).map((tool) => tool.name)).toContain("run_simulation");

    const unpaired = await cli(context, ["run", "--top", "top", "--until", "10"]);
    expect(unpaired.code).toBe(1);
    expect(JSON.parse(unpaired.stdout)).toMatchObject({ code: "BROWSER_DISCONNECTED" });

    const status = JSON.parse((await cli(context, ["status"])).stdout) as typeof startInfo.connection;
    await pairMockBrowser(context, status);
    expect(JSON.parse((await cli(context, ["status"])).stdout)).toMatchObject({ connected: true });

    const localFile = path.join(context.stateDir, "tb.sv");
    await fs.writeFile(localFile, "module tb; endmodule\n");
    const created = await cli(context, ["put", "tb.sv", localFile, "--create"]);
    expect(created.code, created.stdout).toBe(0);
    const read = await cli(context, ["cat", "tb.sv"]);
    expect(read.stdout).toBe("module tb; endmodule\n");
    expect(read.stderr).toContain(`revision ${revision(Buffer.from("module tb; endmodule\n"))}`);

    const stale = await cli(context, ["put", "tb.sv", localFile, "--expected-revision", revision(Buffer.from("old"))]);
    expect(stale.code).toBe(1);
    expect(JSON.parse(stale.stdout)).toMatchObject({ code: "REVISION_CONFLICT" });

    const run = await cli(context, ["run", "--top", "top", "--until", "100", "--unit", "ns"]);
    expect(run.code, run.stdout).toBe(0);
    expect(JSON.parse(run.stdout)).toMatchObject({ status: "finish", stdout: "files=tb.sv,top.sv\n", lastTime: { ticks: "100" } });

    const called = await cli(context, ["call", "read_project_file", '{"path":"../escape.sv"}']);
    expect(called.code).toBe(1);

    const stopped = await cli(context, ["stop"]);
    expect(JSON.parse(stopped.stdout)).toMatchObject({ stopped: true });
    const afterStop = await cli(context, ["status"]);
    expect(afterStop.code).toBe(2);
    expect(afterStop.stderr).toContain("No running hdlwb session");
  });

  it("rejects session requests without the token or from a browser origin", async () => {
    const context = await createContext();
    expect((await cli(context, ["start", "--project-root", context.root, "--port", "0", "--allowed-origins", ORIGIN])).code).toBe(0);
    const state = JSON.parse(await fs.readFile(path.join(context.stateDir, "test.json"), "utf8")) as { port: number; token: string };
    expect(await rawRequest(state.port, {})).toBe(401);
    expect(await rawRequest(state.port, { authorization: "Bearer wrong" })).toBe(401);
    expect(await rawRequest(state.port, { authorization: `Bearer ${state.token}`, origin: ORIGIN })).toBe(403);
    expect(await rawRequest(state.port, { authorization: `Bearer ${state.token}` })).toBe(200);
  });

  it("reports usage errors without starting a session", async () => {
    const context = await createContext();
    const missing = await cli(context, ["start", "--project-root", path.join(context.root, "missing")]);
    expect(missing.code).toBe(2);
    expect(missing.stderr).toContain("not a directory");
    const put = await cli(context, ["put", "top.sv", "-"]);
    expect(put.code).toBe(2);
    expect(put.stderr).toContain("--expected-revision");
  });
});
