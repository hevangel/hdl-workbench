import { test, expect } from "@playwright/test";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(packageRoot, "../..");
const bridgeEntry = path.join(packageRoot, "dist", "main.js");
const dpiFixture = path.join(repositoryRoot, "apps/ide/public/fixtures/dpi");
const browserOrigin = new URL(process.env.XEZIM_PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:5174").origin;

function readToolResult(result) {
  const text = result.content.find((item) => item.type === "text")?.text;
  if (!text) throw new Error("MCP tool returned no JSON text content");
  return JSON.parse(text);
}

function parseToolResult(result) {
  if (result.isError) throw new Error(JSON.stringify(readToolResult(result)));
  return readToolResult(result);
}

async function runDpi(client) {
  return parseToolResult(await client.callTool({
    name: "run_simulation",
    arguments: {
      top_module: "dpi_top",
      until_ticks: "1",
      timescale: { magnitude: "1", unit: "ns" },
      work_budget: { max_wall_time_ms: 20_000, max_output_bytes: 1_048_576, max_vcd_bytes: 64 * 1024 * 1024 },
    },
  }));
}

test("MCP project C/C++ files compile and execute through IDE Xezim DPI with recovery", async ({ page }) => {
  const projectRoot = await mkdtemp(path.join(os.tmpdir(), "xezim-luna15-dpi-e2e-"));
  const fixtureFiles = ["top.sv", "add.c", "add.h", "add_cpp.cpp", "add_cpp.hpp"];
  const destinations = ["dpi_top.sv", "dpi/add.c", "dpi/add.h", "dpi/add_cpp.cpp", "dpi/add_cpp.hpp"];
  for (let index = 0; index < fixtureFiles.length; index += 1) {
    const relativePath = destinations[index];
    const destination = path.join(projectRoot, relativePath);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, await readFile(path.join(dpiFixture, fixtureFiles[index])), "utf8");
  }

  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [bridgeEntry, "--project-root", projectRoot, "--port", "0", "--allowed-origins", browserOrigin, "--request-timeout-ms", "60000"],
    cwd: packageRoot,
    stderr: "pipe",
  });
  const client = new Client({ name: "xezim-studio-luna15-dpi-e2e", version: "0.1.0" });

  try {
    await client.connect(transport);
    await page.goto("/");
    await expect(page.locator(".monaco-editor .view-lines")).toBeVisible();
    await page.getByLabel("Top module").fill("dpi_top");

    const status = parseToolResult(await client.callTool({ name: "get_connection_status", arguments: {} }));
    await page.getByRole("button", { name: "Connect agent" }).click();
    await page.getByLabel("Connection status response").fill(JSON.stringify(status));
    await page.getByRole("button", { name: "Pair browser" }).click();
    await expect(page.locator(".mcp-connection-state")).toHaveText("Connected");

    const successfulRun = await runDpi(client);
    expect(successfulRun.status).toBe("finish");
    expect(successfulRun.stdout).toContain("DPI_C=30");
    expect(successfulRun.stdout).toContain("DPI_CPP=42");

    const files = parseToolResult(await client.callTool({ name: "list_project_files", arguments: {} }));
    const cFile = files.files.find((file) => file.path === "dpi/add.c");
    expect(cFile).toBeDefined();
    const brokenC = "int add_ints( {";
    const damaged = parseToolResult(await client.callTool({
      name: "write_project_file",
      arguments: {
        path: "dpi/add.c",
        expected_revision: cFile.revision,
        content_base64: Buffer.from(brokenC, "utf8").toString("base64"),
      },
    }));

    const failed = await client.callTool({
      name: "run_simulation",
      arguments: {
        top_module: "dpi_top",
        until_ticks: "1",
        timescale: { magnitude: "1", unit: "ns" },
        work_budget: { max_wall_time_ms: 20_000, max_output_bytes: 1_048_576, max_vcd_bytes: 64 * 1024 * 1024 },
      },
    });
    expect(failed.isError).toBe(true);
    expect(readToolResult(failed)).toMatchObject({ code: "DPI_COMPILATION_FAILED" });

    await client.callTool({
      name: "write_project_file",
      arguments: {
        path: "dpi/add.c",
        expected_revision: damaged.revision,
        content_base64: await readFile(path.join(dpiFixture, "add.c"), "base64"),
      },
    });
    const recoveredRun = await runDpi(client);
    expect(recoveredRun.status).toBe("finish");
    expect(recoveredRun.stdout).toContain("DPI_C=30");
    expect(recoveredRun.stdout).toContain("DPI_CPP=42");
  } finally {
    await client.close().catch(() => undefined);
    await transport.close().catch(() => undefined);
    await rm(projectRoot, { recursive: true, force: true });
  }
});
