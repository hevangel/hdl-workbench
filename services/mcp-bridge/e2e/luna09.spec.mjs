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
const counterFixture = path.join(repositoryRoot, "crates/xezim-web/fixtures/counter.sv");
const projectFixture = path.join(repositoryRoot, "crates/xezim-web/fixtures/project");
const projectFixtureFiles = [
  "rtl/marker_pkg.sv",
  "rtl/top.sv",
  "rtl/includes/top_defs.svh",
  "rtl/includes/nested.svh",
  "rom.hex",
];

function parseToolResult(result) {
  if (result.isError) {
    const text = result.content.find((item) => item.type === "text")?.text ?? "MCP tool failed";
    throw new Error(text);
  }
  const text = result.content.find((item) => item.type === "text")?.text;
  if (!text) throw new Error("MCP tool returned no JSON text content");
  return JSON.parse(text);
}

async function expectAgentEditVisible(page) {
  const editor = page.locator(".monaco-editor .view-lines");
  await editor.click();
  await page.keyboard.press("Control+End");
  await expect(editor).toContainText("edited through authenticated MCP in LUNA-09");
}

test("MCP edit -> Xezim WASM counter and ROM runs -> Surfer VCD -> workspace survives reload", async ({ page }) => {
  const projectRoot = await mkdtemp(path.join(os.tmpdir(), "xezim-luna09-e2e-"));
  const fixtureSource = await readFile(counterFixture, "utf8");
  const initialSource = fixtureSource.replace(
    "  initial begin",
    '  initial begin\n    $dumpfile("counter.vcd");\n    $dumpvars(0, counter);',
  );
  await writeFile(path.join(projectRoot, "counter.sv"), initialSource, "utf8");
  for (const relativePath of projectFixtureFiles) {
    const destination = path.join(projectRoot, relativePath);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, await readFile(path.join(projectFixture, relativePath)), "utf8");
  }
  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [bridgeEntry, "--project-root", projectRoot, "--port", "0", "--allowed-origins", "http://127.0.0.1:5174", "--request-timeout-ms", "60000"],
    cwd: packageRoot,
    stderr: "pipe",
  });
  const client = new Client({ name: "xezim-studio-luna09-e2e", version: "0.1.0" });

  try {
    await client.connect(transport);
    await page.goto("/");
    await expect(page.locator(".monaco-editor .view-lines")).toBeVisible();
    await expect(page.locator(".surfer-readiness.ready")).toHaveText("Surfer v0.7.0 ready");
    await page.getByLabel("Top module").fill("counter");

    const status = parseToolResult(await client.callTool({ name: "get_connection_status", arguments: {} }));
    await page.getByRole("button", { name: "Connect agent" }).click();
    await page.getByLabel("Connection status response").fill(JSON.stringify(status));
    await page.getByRole("button", { name: "Pair browser" }).click();
    await expect(page.locator(".mcp-connection-state")).toHaveText("Connected");

    const listed = parseToolResult(await client.callTool({ name: "list_project_files", arguments: {} }));
    const prior = listed.files.find((file) => file.path === "counter.sv");
    const editedSource = `${initialSource}\n// edited through authenticated MCP in LUNA-09\n`;
    parseToolResult(await client.callTool({
      name: "write_project_file",
      arguments: {
        path: "counter.sv",
        expected_revision: prior.revision,
        content_base64: Buffer.from(editedSource, "utf8").toString("base64"),
      },
    }));

    const run = parseToolResult(await client.callTool({
      name: "run_simulation",
      arguments: {
        top_module: "counter",
        until_ticks: "100",
        timescale: { magnitude: "1", unit: "ns" },
        work_budget: { max_wall_time_ms: 20_000, max_output_bytes: 1_048_576, max_vcd_bytes: 64 * 1024 * 1024 },
      },
    }));
    expect(run.status).toBe("finish");
    expect(run.stdout).toContain("COUNT=4");
    expect(run.waveform).toMatchObject({ format: "vcd", complete: true, storage: "opfs" });

    const romRun = parseToolResult(await client.callTool({
      name: "run_simulation",
      arguments: {
        top_module: "top",
        until_ticks: "20",
        timescale: { magnitude: "1", unit: "ns" },
        work_budget: { max_wall_time_ms: 20_000, max_output_bytes: 1_048_576, max_vcd_bytes: 64 * 1024 * 1024 },
      },
    }));
    expect(romRun.status).toBe("finish");
    expect(romRun.stdout).toContain("FILE=ROM=f0");
    expect(romRun.stdout).toContain("NBA_PRE=1");
    expect(romRun.stdout).toContain("NBA_POST=2");
    expect(romRun.stdout).toContain("ROM=f0 COUNT=3");
    expect(romRun.waveform).toMatchObject({ format: "vcd", complete: true, storage: "opfs" });

    await expect(page.getByRole("button", { name: "Open counter.sv" })).toBeVisible();
    await page.getByRole("button", { name: "Open counter.sv" }).click();
    await expectAgentEditVisible(page);

    parseToolResult(await client.callTool({
      name: "show_waveforms",
      arguments: { artifact_id: run.waveform.id, signal_paths: ["counter.count"], cursor: { ticks: "25" } },
    }));
    await expect(page.getByRole("tab", { name: "WAVEFORMS" })).toHaveAttribute("aria-selected", "true");
    const surferNotice = page.getByLabel("Surfer license and source information");
    await expect(surferNotice).toContainText("EUPL-1.2 · modified by Xezim Studio on 2026-10-22");
    await expect(surferNotice.getByRole("link", { name: "License" })).toHaveAttribute("href", "/surfer/LICENSE-EUPL-1.2.txt");
    await expect(surferNotice.getByRole("link", { name: "Pinned source and changes" })).toHaveAttribute("href", "/surfer/UPSTREAM.md");
    const [licenseResponse, sourceResponse] = await Promise.all([
      page.request.get("/surfer/LICENSE-EUPL-1.2.txt"),
      page.request.get("/surfer/UPSTREAM.md"),
    ]);
    expect(licenseResponse.ok()).toBe(true);
    expect(await licenseResponse.text()).toContain("EUROPEAN UNION PUBLIC LICENCE v. 1.2");
    expect(sourceResponse.ok()).toBe(true);
    expect(await sourceResponse.text()).toContain("bd749b1f786c1c62cd67893ca71346cbe6983915");
    const surfer = page.frameLocator("#surferFrame");
    await expect(surfer.locator("canvas")).toHaveCount(1);

    await page.reload();
    await expect(page.getByRole("button", { name: "Open counter.sv" })).toBeVisible();
    await page.getByRole("button", { name: "Open counter.sv" }).click();
    await expectAgentEditVisible(page);
    await expect(page.locator(".surfer-readiness.ready")).toHaveText("Surfer v0.7.0 ready");

    const reconnectedStatus = parseToolResult(await client.callTool({ name: "get_connection_status", arguments: {} }));
    await page.getByRole("button", { name: "Connect agent" }).click();
    await page.getByLabel("Connection status response").fill(JSON.stringify(reconnectedStatus));
    await page.getByRole("button", { name: "Pair browser" }).click();
    await expect(page.locator(".mcp-connection-state")).toHaveText("Connected");
    parseToolResult(await client.callTool({
      name: "show_waveforms",
      arguments: { artifact_id: run.waveform.id, signal_paths: ["counter.count"], cursor: { ticks: "25" } },
    }));
    await expect(page.getByRole("tab", { name: "WAVEFORMS" })).toHaveAttribute("aria-selected", "true");
    await expect(page.frameLocator("#surferFrame").locator("canvas")).toHaveCount(1);
  } finally {
    await client.close().catch(() => undefined);
    await transport.close().catch(() => undefined);
    await rm(projectRoot, { recursive: true, force: true });
  }
});
