#!/usr/bin/env node
// hdlwb: a persistent command-line session for the HDL Workbench MCP bridge.
//
// MCP stdio clients such as the Inspector CLI start a new bridge for every call,
// which drops the browser pairing. `hdlwb start` keeps one bridge and one MCP
// client alive in a background process; later commands forward tool calls to it
// over a loopback HTTP channel protected by a random token. The session process
// only relays the bridge's MCP tools; it never runs commands or a simulator.
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { spawn } from "node:child_process";
import { randomBytes, timingSafeEqual } from "node:crypto";
import { closeSync, existsSync, openSync, readFileSync } from "node:fs";
import { mkdir, readFile, rename, rm, stat, writeFile } from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const packageRoot = path.resolve(path.dirname(scriptPath), "..");
const bridgeEntry = path.join(packageRoot, "dist", "main.js");
const stateDirectory = process.env.HDLWB_STATE_DIR ?? path.join(os.tmpdir(), "hdl-workbench-cli");
const maxBodyBytes = 32 * 1024 * 1024;
const toolTimeoutMs = 330_000;
const startupTimeoutMs = 20_000;

const usage = `Usage: hdlwb <command> [options]

Session
  start --project-root <dir> [--port <n>] [--allowed-origins <a,b>] [--request-timeout-ms <n>]
                          Start the bridge in the background and print the pairing JSON
  status                  Print get_connection_status (pairing JSON and "connected")
  stop                    Stop the session and its bridge

Tools
  tools                   List the bridge's MCP tools
  call <tool> [json|@file|-]
                          Call any tool with JSON arguments (default {})
  run --top <module> --until <ticks> [--unit ns] [--magnitude 1]
      [--wall-ms <n>] [--max-output-bytes <n>] [--max-vcd-bytes <n>]
                          run_simulation in the paired browser
  cat <path>              Print a project file's content (revision goes to stderr)
  put <path> <file|-> (--expected-revision <rev> | --create)
                          write_project_file from a local file or stdin

Every command accepts --session <name> (or HDLWB_SESSION) to run several sessions.
Exit codes: 0 success, 1 tool error (JSON on stdout), 2 usage or session error.`;

class UsageError extends Error {}

function parseOptions(args, allowedFlags = [], booleanFlags = []) {
  const options = {};
  const positional = [];
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (!arg.startsWith("--") || arg === "-") {
      positional.push(arg);
      continue;
    }
    const name = arg.slice(2);
    if (booleanFlags.includes(name)) {
      options[name] = true;
      continue;
    }
    if (name !== "session" && !allowedFlags.includes(name)) throw new UsageError(`Unknown option --${name}`);
    const value = args[index + 1];
    if (value === undefined || value.startsWith("--")) throw new UsageError(`Missing value for --${name}`);
    options[name] = value;
    index += 1;
  }
  return { options, positional };
}

function sessionName(options) {
  const name = options.session ?? process.env.HDLWB_SESSION ?? "default";
  if (!/^[A-Za-z0-9_-]{1,64}$/.test(name)) throw new UsageError("Session names use 1-64 letters, digits, '-' or '_'");
  return name;
}

function sessionPaths(name) {
  return { state: path.join(stateDirectory, `${name}.json`), log: path.join(stateDirectory, `${name}.log`) };
}

function print(value) {
  process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
}

// ---------------------------------------------------------------------------
// Client side: talk to a running session over loopback HTTP.

async function readState(name) {
  try {
    return JSON.parse(await readFile(sessionPaths(name).state, "utf8"));
  } catch (error) {
    if (error?.code === "ENOENT") return null;
    throw error;
  }
}

function request(state, method, route, body) {
  const payload = body === undefined ? undefined : Buffer.from(JSON.stringify(body));
  return new Promise((resolve, reject) => {
    const req = http.request({
      host: "127.0.0.1",
      port: state.port,
      method,
      path: route,
      headers: {
        authorization: `Bearer ${state.token}`,
        ...(payload ? { "content-type": "application/json", "content-length": payload.length } : {}),
      },
    }, (res) => {
      const chunks = [];
      res.on("data", (chunk) => chunks.push(chunk));
      res.on("end", () => {
        const text = Buffer.concat(chunks).toString("utf8");
        let parsed;
        try { parsed = JSON.parse(text); } catch { parsed = { error: text }; }
        if (res.statusCode !== 200) reject(new Error(parsed.error ?? `Session returned HTTP ${res.statusCode}`));
        else resolve(parsed);
      });
    });
    req.on("error", reject);
    req.end(payload);
  });
}

async function liveState(name) {
  const state = await readState(name);
  if (!state) return null;
  try {
    await request(state, "GET", "/health");
    return state;
  } catch {
    await rm(sessionPaths(name).state, { force: true });
    return null;
  }
}

async function requireSession(name) {
  const state = await liveState(name);
  if (!state) throw new UsageError(`No running hdlwb session "${name}". Start one with: hdlwb start --project-root <dir>`);
  return state;
}

async function callTool(state, tool, args = {}) {
  return request(state, "POST", "/call", { tool, arguments: args });
}

// Prints a tool result and returns the process exit code.
function emitToolResult(result) {
  print(result.value);
  return result.isError ? 1 : 0;
}

async function readInput(source) {
  if (source === "-") {
    const chunks = [];
    for await (const chunk of process.stdin) chunks.push(chunk);
    return Buffer.concat(chunks);
  }
  return readFile(source);
}

async function parseArguments(source) {
  if (source === undefined) return {};
  const text = source === "-" || source.startsWith("@")
    ? (await readInput(source === "-" ? "-" : source.slice(1))).toString("utf8")
    : source;
  try {
    return JSON.parse(text);
  } catch {
    throw new UsageError("Tool arguments must be a JSON object, @file or - for stdin");
  }
}

function positiveInteger(value, flag) {
  if (value === undefined) return undefined;
  if (!/^\d+$/.test(value)) throw new UsageError(`--${flag} must be a non-negative integer`);
  return Number(value);
}

async function start(args) {
  const { options } = parseOptions(args, ["project-root", "port", "allowed-origins", "request-timeout-ms"]);
  const name = sessionName(options);
  const projectRoot = options["project-root"] ?? process.env.XEZIM_PROJECT_ROOT;
  if (!projectRoot) throw new UsageError("start needs --project-root <dir>");
  const resolvedRoot = path.resolve(projectRoot);
  if (!(await stat(resolvedRoot).catch(() => null))?.isDirectory()) throw new UsageError(`Project root is not a directory: ${resolvedRoot}`);
  if (!existsSync(bridgeEntry)) throw new UsageError(`Bridge is not built (${bridgeEntry} missing). Run: pnpm --filter @xezim-ide/mcp-bridge build`);

  const existing = await liveState(name);
  if (existing) throw new UsageError(`Session "${name}" is already running for ${existing.projectRoot}. Run hdlwb stop first.`);

  const paths = sessionPaths(name);
  await mkdir(stateDirectory, { recursive: true });
  await rm(paths.state, { force: true });
  const bridgeArgs = ["--project-root", resolvedRoot, "--request-timeout-ms", options["request-timeout-ms"] ?? "300000"];
  if (options.port) bridgeArgs.push("--port", options.port);
  if (options["allowed-origins"]) bridgeArgs.push("--allowed-origins", options["allowed-origins"]);

  const logFd = openSync(paths.log, "w");
  const child = spawn(process.execPath, [scriptPath, "__session", name, JSON.stringify(bridgeArgs)], {
    cwd: packageRoot,
    detached: true,
    stdio: ["ignore", logFd, logFd],
    windowsHide: true,
    env: { ...process.env, HDLWB_STATE_DIR: stateDirectory },
  });
  closeSync(logFd);
  let exited = false;
  child.once("exit", () => { exited = true; });

  const deadline = Date.now() + startupTimeoutMs;
  let state = null;
  while (!state && !exited && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 100));
    state = await readState(name).catch(() => null);
  }
  if (!state) {
    if (!exited) child.kill();
    const log = existsSync(paths.log) ? readFileSync(paths.log, "utf8").trim().split("\n").slice(-10).join("\n") : "";
    throw new UsageError(`Session failed to start.${log ? `\n${log}` : ""}`);
  }
  child.unref();
  const status = await callTool(state, "get_connection_status");
  print({ session: name, projectRoot: state.projectRoot, log: paths.log, connection: status.value });
  return status.isError ? 1 : 0;
}

async function stop(args) {
  const { options } = parseOptions(args);
  const name = sessionName(options);
  const state = await liveState(name);
  if (!state) {
    print({ session: name, stopped: false, reason: "not running" });
    return 0;
  }
  await request(state, "POST", "/stop").catch(() => undefined);
  await rm(sessionPaths(name).state, { force: true });
  print({ session: name, stopped: true });
  return 0;
}

async function run(args) {
  const { options } = parseOptions(args, ["top", "until", "unit", "magnitude", "wall-ms", "max-output-bytes", "max-vcd-bytes"]);
  if (!options.top || !options.until) throw new UsageError("run needs --top <module> and --until <ticks>");
  if (!/^\d+$/.test(options.until)) throw new UsageError("--until is a whole number of ticks; set the unit with --unit");
  const budget = [options["wall-ms"], options["max-output-bytes"], options["max-vcd-bytes"]];
  const runArgs = {
    top_module: options.top,
    until_ticks: options.until,
    timescale: { magnitude: options.magnitude ?? "1", unit: options.unit ?? "ns" },
  };
  if (budget.some((value) => value !== undefined)) {
    runArgs.work_budget = {
      max_wall_time_ms: positiveInteger(options["wall-ms"], "wall-ms") ?? 30_000,
      max_output_bytes: positiveInteger(options["max-output-bytes"], "max-output-bytes") ?? 1_048_576,
      max_vcd_bytes: positiveInteger(options["max-vcd-bytes"], "max-vcd-bytes") ?? 16 * 1024 * 1024,
    };
  }
  const state = await requireSession(sessionName(options));
  return emitToolResult(await callTool(state, "run_simulation", runArgs));
}

async function cat(args) {
  const { options, positional } = parseOptions(args);
  if (positional.length !== 1) throw new UsageError("cat needs exactly one project path");
  const state = await requireSession(sessionName(options));
  const result = await callTool(state, "read_project_file", { path: positional[0] });
  if (result.isError) return emitToolResult(result);
  process.stderr.write(`revision ${result.value.revision}\n`);
  process.stdout.write(Buffer.from(result.value.contentBase64, "base64"));
  return 0;
}

async function put(args) {
  const { options, positional } = parseOptions(args, ["expected-revision"], ["create"]);
  if (positional.length !== 2) throw new UsageError("put needs <project-path> and <local-file|->");
  if (Boolean(options.create) === Boolean(options["expected-revision"])) {
    throw new UsageError("put needs exactly one of --expected-revision <rev> (replace) or --create (new file)");
  }
  const state = await requireSession(sessionName(options));
  const content = await readInput(positional[1]);
  return emitToolResult(await callTool(state, "write_project_file", {
    path: positional[0],
    expected_revision: options.create ? null : options["expected-revision"],
    content_base64: content.toString("base64"),
  }));
}

async function main(argv) {
  const [command, ...args] = argv;
  switch (command) {
    case "start": return start(args);
    case "stop": return stop(args);
    case "status": {
      const { options } = parseOptions(args);
      return emitToolResult(await callTool(await requireSession(sessionName(options)), "get_connection_status"));
    }
    case "tools": {
      const { options } = parseOptions(args);
      print((await request(await requireSession(sessionName(options)), "GET", "/tools")).tools);
      return 0;
    }
    case "call": {
      const { options, positional } = parseOptions(args);
      if (positional.length < 1 || positional.length > 2) throw new UsageError("call needs <tool> and optional JSON arguments");
      const toolArgs = await parseArguments(positional[1]);
      return emitToolResult(await callTool(await requireSession(sessionName(options)), positional[0], toolArgs));
    }
    case "run": return run(args);
    case "cat": return cat(args);
    case "put": return put(args);
    case "__session": return session(args[0], JSON.parse(args[1]));
    case undefined:
    case "help":
    case "--help":
    case "-h":
      process.stdout.write(`${usage}\n`);
      return command === undefined ? 2 : 0;
    default:
      throw new UsageError(`Unknown command "${command}"\n\n${usage}`);
  }
}

// ---------------------------------------------------------------------------
// Session side: own the bridge and its MCP client, serve tool calls on loopback.

function parseToolResult(result) {
  const text = result.content?.find((item) => item.type === "text")?.text;
  let value = text;
  try { value = text === undefined ? null : JSON.parse(text); } catch { /* keep the raw text */ }
  return { isError: Boolean(result.isError), value };
}

function tokenMatches(header, token) {
  const expected = Buffer.from(`Bearer ${token}`);
  const actual = Buffer.from(header ?? "");
  return actual.length === expected.length && timingSafeEqual(actual, expected);
}

async function session(name, bridgeArgs) {
  const paths = sessionPaths(name);
  const log = (message) => process.stderr.write(`[hdlwb] ${new Date().toISOString()} ${message}\n`);
  const logFd = openSync(paths.log, "a");
  const transport = new StdioClientTransport({ command: process.execPath, args: [bridgeEntry, ...bridgeArgs], cwd: packageRoot, stderr: logFd });
  const client = new Client({ name: "hdl-workbench-cli", version: "0.1.0" });
  const token = randomBytes(32).toString("hex");
  let server;
  let stopping = false;

  const shutdown = async (reason) => {
    if (stopping) return;
    stopping = true;
    log(`stopping: ${reason}`);
    const state = await readState(name).catch(() => null);
    if (state?.pid === process.pid) await rm(paths.state, { force: true });
    server?.close();
    await client.close().catch(() => undefined);
    process.exit(0);
  };

  transport.onclose = () => { void shutdown("bridge exited"); };
  await client.connect(transport);
  const projectRoot = bridgeArgs[bridgeArgs.indexOf("--project-root") + 1];

  server = http.createServer(async (req, res) => {
    const reply = (status, body) => {
      res.writeHead(status, { "content-type": "application/json" });
      res.end(JSON.stringify(body));
    };
    // Browsers always send Origin on cross-site requests; the CLI never does.
    if (req.headers.origin !== undefined || req.headers.host !== `127.0.0.1:${server.address().port}`) return reply(403, { error: "Forbidden" });
    if (!tokenMatches(req.headers.authorization, token)) return reply(401, { error: "Unauthorized" });
    try {
      if (req.method === "GET" && req.url === "/health") return reply(200, { ok: true });
      if (req.method === "GET" && req.url === "/tools") {
        const { tools } = await client.listTools();
        return reply(200, { tools: tools.map(({ name: toolName, description }) => ({ name: toolName, description })) });
      }
      if (req.method === "POST" && req.url === "/stop") {
        reply(200, { ok: true });
        return void shutdown("stop requested");
      }
      if (req.method === "POST" && req.url === "/call") {
        const chunks = [];
        let size = 0;
        for await (const chunk of req) {
          size += chunk.length;
          if (size > maxBodyBytes) return reply(413, { error: "Request body too large" });
          chunks.push(chunk);
        }
        const body = JSON.parse(Buffer.concat(chunks).toString("utf8"));
        if (typeof body.tool !== "string" || typeof body.arguments !== "object" || body.arguments === null || Array.isArray(body.arguments)) {
          return reply(400, { error: "Expected { tool, arguments }" });
        }
        log(`call ${body.tool}`);
        const result = await client.callTool({ name: body.tool, arguments: body.arguments }, { timeout: toolTimeoutMs });
        return reply(200, parseToolResult(result));
      }
      return reply(404, { error: "Not found" });
    } catch (error) {
      return reply(500, { error: error instanceof Error ? error.message : String(error) });
    }
  });
  server.requestTimeout = 0;
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));

  const state = { pid: process.pid, port: server.address().port, token, projectRoot, startedAt: new Date().toISOString(), log: paths.log };
  const temporary = `${paths.state}.${process.pid}.tmp`;
  await writeFile(temporary, JSON.stringify(state), { mode: 0o600 });
  await rename(temporary, paths.state);
  log(`session "${name}" ready on 127.0.0.1:${state.port} for ${projectRoot}`);
  process.once("SIGINT", () => { void shutdown("SIGINT"); });
  process.once("SIGTERM", () => { void shutdown("SIGTERM"); });
  return new Promise(() => {});
}

main(process.argv.slice(2)).then(
  (code) => { process.exitCode = code; },
  (error) => {
    process.stderr.write(`hdlwb: ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 2;
  },
);
