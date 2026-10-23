import path from "node:path";
import { fileURLToPath } from "node:url";
import { serveStdio } from "@modelcontextprotocol/server/stdio";
import { ProjectFiles } from "./projectFiles.js";
import { createMcpServer } from "./mcp.js";
import { IdeMcpService } from "./service.js";
import { WebSocketBridge } from "./websocketBridge.js";

interface CliOptions {
  projectRoot: string;
  port: number;
  origins: string[];
  requestTimeoutMs: number;
}

function parseArgs(args: string[]): CliOptions {
  const values = new Map<string, string>();
  for (let index = 0; index < args.length; index += 1) {
    const key = args[index];
    if (!key?.startsWith("--")) throw new Error(`Unexpected argument: ${key ?? ""}`);
    const value = args[index + 1];
    if (!value || value.startsWith("--")) throw new Error(`Missing value for ${key}`);
    values.set(key, value);
    index += 1;
  }
  const projectRoot = values.get("--project-root") ?? process.env.XEZIM_PROJECT_ROOT ?? "";
  const port = Number(values.get("--port") ?? process.env.XEZIM_BRIDGE_PORT ?? "47531");
  const requestTimeoutMs = Number(values.get("--request-timeout-ms") ?? process.env.XEZIM_BRIDGE_TIMEOUT_MS ?? "30000");
  const origins = (values.get("--allowed-origins") ?? process.env.XEZIM_ALLOWED_ORIGINS ?? "http://localhost:5173,http://127.0.0.1:5173")
    .split(",").map((origin) => origin.trim()).filter(Boolean);
  if (!projectRoot) throw new Error("Set XEZIM_PROJECT_ROOT or pass --project-root <directory>; the bridge has no implicit workspace");
  if (!Number.isInteger(port) || port < 0 || port > 65535) throw new Error("--port must be an integer between 0 and 65535");
  if (!Number.isInteger(requestTimeoutMs) || requestTimeoutMs < 1 || requestTimeoutMs > 300_000) throw new Error("--request-timeout-ms must be between 1 and 300000");
  return { projectRoot, port, origins, requestTimeoutMs };
}

async function start(): Promise<void> {
  const options = parseArgs(process.argv.slice(2));
  const files = await ProjectFiles.open(options.projectRoot);
  const bridge = new WebSocketBridge({ allowedOrigins: options.origins, port: options.port, requestTimeoutMs: options.requestTimeoutMs, logger: (message) => console.error(`[xezim-bridge] ${message}`) });
  await bridge.start();
  const service = new IdeMcpService(files, bridge);
  process.once("SIGINT", () => { void bridge.close(); });
  process.once("SIGTERM", () => { void bridge.close(); });
  console.error(`[xezim-bridge] MCP stdio ready; project root: ${files.root}; browser URL: ${bridge.url}`);
  void serveStdio(() => createMcpServer(service));
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : "";
if (invokedPath === fileURLToPath(import.meta.url)) {
  void start().catch((error: unknown) => {
    const message = error instanceof Error ? error.message : "Unexpected startup error";
    console.error(`[xezim-bridge] ${message}`);
    process.exitCode = 1;
  });
}
