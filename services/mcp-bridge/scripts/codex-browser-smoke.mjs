import { spawn } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { chromium, expect } from "@playwright/test";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(packageRoot, "../..");
const appRoot = path.join(repositoryRoot, "apps/ide");
const viteRequire = createRequire(path.join(appRoot, "package.json"));
const viteCli = path.join(path.dirname(viteRequire.resolve("vite/package.json")), "bin", "vite.js");
const bridgeEntry = path.join(packageRoot, "dist", "main.js");
const origin = "http://127.0.0.1";
const editedSource = '`timescale 1ns/1ps\nmodule top; initial begin $dumpfile("codex-smoke.vcd"); $dumpvars(0, top); $display("CODEX_EDITED=OK"); $finish; end endmodule\n';
const contentBase64 = Buffer.from(editedSource, "utf8").toString("base64");
const timeoutMs = 180_000;

function reservePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        server.close();
        reject(new Error("Could not reserve a local Vite port."));
        return;
      }
      server.close((error) => error ? reject(error) : resolve(address.port));
    });
  });
}

function resultObject(item) {
  if (item.result?.isError) throw new Error(`Codex MCP call ${item.tool} returned an error.`);
  if (item.result?.structuredContent && typeof item.result.structuredContent === "object") {
    return item.result.structuredContent;
  }
  const text = item.result?.content?.find((block) => block.type === "text")?.text;
  if (!text) throw new Error(`Codex MCP call ${item.tool} returned no JSON text.`);
  try {
    return JSON.parse(text);
  } catch {
    const prefix = text.slice(0, 160).replace(/[\r\n\t]+/g, " ");
    throw new Error(`Codex MCP call ${item.tool} returned non-JSON text (${text.length} chars): ${JSON.stringify(prefix)}`);
  }
}

async function waitForVite(child, url) {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error("Vite exited before the IDE became ready.");
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Vite is still starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error("Vite did not become ready within 60 seconds.");
}

const projectRoot = await mkdtemp(path.join(os.tmpdir(), "xezim-codex-mcp-smoke-"));
let vite;
let browser;
let codex;
let killTimer;
let eventBuffer = "";
let eventQueue = Promise.resolve();
let eventError;
let paired = false;
let wroteFile = false;
let simulation;
let simulationShape = "missing";
const calledTools = [];

try {
  await writeFile(path.join(projectRoot, "top.sv"), "`timescale 1ns/1ps\nmodule top; initial begin $dumpfile(\"codex-smoke.vcd\"); $dumpvars(0, top); $display(\"BEFORE\"); $finish; end endmodule\n", "utf8");

  const port = await reservePort();
  const baseUrl = `${origin}:${port}`;
  vite = spawn(process.execPath, [viteCli, "--host", "127.0.0.1", "--port", String(port), "--strictPort"], {
    cwd: appRoot,
    windowsHide: true,
    stdio: ["ignore", "ignore", "ignore"],
  });
  await waitForVite(vite, baseUrl);

  browser = await chromium.launch({
    headless: true,
    args: ["--enable-webgl", "--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
  });
  const page = await browser.newPage();
  await page.goto(baseUrl);
  await page.locator(".monaco-editor .view-lines").waitFor({ timeout: 30_000 });

  const codexCommand = process.env.XEZIM_CODEX_CLI || (process.platform === "win32" ? "codex.exe" : "codex");
  const serverArgs = [
    bridgeEntry.replaceAll("\\", "/"),
    "--project-root", projectRoot.replaceAll("\\", "/"),
    "--port", "0",
    "--allowed-origins", baseUrl,
    "--request-timeout-ms", "60000",
  ];
  const prompt = [
    "Use only the MCP server named xezim. Do not run shell commands or call tools from other servers.",
    "First call get_connection_status. The local browser will pair automatically after that result is returned.",
    "Then list_project_files, find top.sv and its current revision, and call write_project_file for top.sv with that expected_revision and this exact content_base64:",
    contentBase64,
    'Finally call run_simulation with top_module "top", until_ticks "100", timescale { magnitude: "1", unit: "ns" }, and work_budget { max_wall_time_ms: 20000, max_output_bytes: 1048576, max_vcd_bytes: 65536 }.',
    "Return only the run status and whether stdout contains CODEX_EDITED=OK. Do not reveal the pairing code.",
  ].join("\n");
  codex = spawn(codexCommand, [
    "exec", "--ephemeral", "--json", "--sandbox", "read-only", "--cd", repositoryRoot,
    "-c", "mcp_servers.xezim.command='node'",
    "-c", `mcp_servers.xezim.args=${JSON.stringify(serverArgs)}`,
    prompt,
  ], { cwd: repositoryRoot, windowsHide: true, stdio: ["ignore", "pipe", "ignore"] });

  const handleEvent = async (event) => {
    if (event?.type !== "item.completed" || event.item?.type !== "mcp_tool_call" || event.item.server !== "xezim") return;
    const item = event.item;
    calledTools.push(item.tool);
    const value = resultObject(item);
    if (item.tool === "get_connection_status") {
      if (value.connected !== false || typeof value.websocketUrl !== "string") {
        throw new Error("Codex MCP status did not return a disconnected browser and WebSocket URL.");
      }
      await page.getByRole("button", { name: "Connect agent" }).click();
      await page.getByLabel("Connection status response").fill(JSON.stringify(value));
      await page.getByRole("button", { name: "Pair browser" }).click();
      await expect(page.locator(".mcp-connection-state")).toHaveText("Connected", { timeout: 15_000 });
      paired = true;
    } else if (item.tool === "write_project_file") {
      wroteFile = true;
    } else if (item.tool === "run_simulation") {
      simulation = value.result && typeof value.result === "object" ? value.result : value;
      simulationShape = `keys=${Object.keys(value).join(",")}, code=${value.code ?? "none"}, message=${String(value.message ?? "").slice(0, 160)}`;
    }
  };

  codex.stdout.on("data", (chunk) => {
    eventBuffer += chunk.toString("utf8");
    const lines = eventBuffer.split(/\r?\n/);
    eventBuffer = lines.pop() ?? "";
    for (const line of lines) {
      if (!line.trim()) continue;
      try {
        const event = JSON.parse(line);
        eventQueue = eventQueue.then(() => handleEvent(event)).catch((error) => {
          eventError ??= error;
          codex.kill();
        });
      } catch {
        eventError ??= new Error("Codex CLI emitted an invalid JSON event.");
        codex.kill();
      }
    }
  });
  killTimer = setTimeout(() => codex.kill(), timeoutMs);

  const exitCode = await new Promise((resolve, reject) => {
    codex.once("error", reject);
    codex.once("close", (code) => resolve(code));
  });
  await eventQueue;
  if (eventError) throw eventError;
  if (exitCode !== 0) throw new Error(`Codex CLI exited with status ${exitCode}.`);
  if (!paired || !wroteFile || simulation?.status !== "finish" || !simulation.stdout?.includes("CODEX_EDITED=OK")) {
    throw new Error(`Codex smoke incomplete: paired=${paired}, wrote=${wroteFile}, tools=${calledTools.join(",")}, run_status=${simulation?.status ?? "missing"}, expected_output=${Boolean(simulation?.stdout?.includes("CODEX_EDITED=OK"))}, run_result=${simulationShape}.`);
  }

  process.stdout.write("Codex CLI MCP smoke passed: pairing, source edit, browser simulation, and expected output.\n");
} catch (error) {
  process.stderr.write(`Codex CLI MCP smoke failed: ${error instanceof Error ? error.message : "unknown error"}\n`);
  process.exitCode = 1;
} finally {
  clearTimeout(killTimer);
  if (codex && codex.exitCode === null) codex.kill();
  if (vite && vite.exitCode === null) vite.kill();
  await browser?.close().catch(() => undefined);
  await new Promise((resolve) => setTimeout(resolve, 150));
  const tempRoot = path.resolve(os.tmpdir());
  const relativeProjectRoot = path.relative(tempRoot, path.resolve(projectRoot));
  if (relativeProjectRoot && !relativeProjectRoot.startsWith(`..${path.sep}`) && relativeProjectRoot !== ".." && !path.isAbsolute(relativeProjectRoot)) {
    await rm(projectRoot, { recursive: true, force: true });
  }
}
