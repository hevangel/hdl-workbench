import { execFileSync, spawn } from "node:child_process";
import { createRequire } from "node:module";
import { readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, firefox } from "@playwright/test";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../../..");
const appRoot = path.join(repositoryRoot, "apps", "ide");
const appRequire = createRequire(path.join(appRoot, "package.json"));
const { createServer } = appRequire("vite");
const browserSelection = process.argv.slice(2).find((argument) => argument !== "--") ?? "both";
const browserNames = browserSelection === "both" ? ["chromium", "firefox"] : [browserSelection];
const allowedBrowserNames = new Set(["chromium", "firefox"]);
if (browserNames.some((name) => !allowedBrowserNames.has(name))) {
  throw new Error("Usage: node scripts/profile-browser-memory.mjs [chromium|firefox|both]");
}

const ramProject = (wordCount) => `module top;
  logic [31:0] ram [0:${wordCount - 1}];
  initial begin
    ram[0] = 32'd1;
    $display("RAM0=%0d", ram[0]);
    $finish;
  end
endmodule
`;

function powershellJson(script) {
  const output = execFileSync("powershell.exe", ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script], {
    encoding: "utf8",
    windowsHide: true,
  }).trim();
  return output ? JSON.parse(output) : [];
}

function getWindowsMemoryForPids(pids) {
  const normalizedPids = [...new Set(pids)].filter(Number.isInteger);
  if (normalizedPids.length === 0) return [];
  const script = `$pids = @(${normalizedPids.join(",")}); $rows = @(Get-Process -Id $pids -ErrorAction SilentlyContinue | Select-Object Id,WorkingSet64,PrivateMemorySize64); ConvertTo-Json -InputObject $rows -Compress`;
  const rows = powershellJson(script);
  return (Array.isArray(rows) ? rows : [rows]).map((row) => ({
    pid: Number(row.Id),
    residentBytes: Number(row.WorkingSet64),
    privateBytes: Number(row.PrivateMemorySize64),
  }));
}

function getWindowsFirefoxPids(rootPid) {
  const script = `$root = ${rootPid}; $rows = @(Get-CimInstance Win32_Process -Filter "Name='firefox.exe'" | Select-Object ProcessId,ParentProcessId); $ids = [System.Collections.Generic.List[int]]::new(); $ids.Add($root); do { $children = @($rows | Where-Object { $ids.Contains([int]$_.ParentProcessId) -and -not $ids.Contains([int]$_.ProcessId) } | ForEach-Object { [int]$_.ProcessId }); foreach ($child in $children) { $ids.Add($child) } } while ($children.Count -gt 0); ConvertTo-Json -InputObject @($ids.ToArray()) -Compress`;
  const result = powershellJson(script);
  return (Array.isArray(result) ? result : [result]).map(Number).filter(Number.isInteger);
}

function startWindowsMemorySampler(pids) {
  const normalizedPids = [...new Set(pids)].filter(Number.isInteger);
  if (normalizedPids.length === 0) throw new Error("Cannot sample an empty browser process set.");
  const script = `$pids = @(${normalizedPids.join(",")}); $watch = [System.Diagnostics.Stopwatch]::StartNew(); while ($true) { $rows = @(Get-Process -Id $pids -ErrorAction SilentlyContinue); if ($rows.Count -gt 0) { [pscustomobject]@{ elapsedMs=$watch.ElapsedMilliseconds; processCount=$rows.Count; residentBytes=[long](($rows | Measure-Object -Property WorkingSet64 -Sum).Sum); privateBytes=[long](($rows | Measure-Object -Property PrivateMemorySize64 -Sum).Sum) } | ConvertTo-Json -Compress }; Start-Sleep -Milliseconds 50 }`;
  const child = spawn("powershell.exe", ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script], {
    windowsHide: true,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const samples = [];
  let buffered = "";
  let stderr = "";
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    buffered += chunk;
    const lines = buffered.split(/\r?\n/);
    buffered = lines.pop() ?? "";
    for (const line of lines) {
      if (line.trim()) samples.push(JSON.parse(line));
    }
  });
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => { stderr = `${stderr}${chunk}`.slice(-2000); });
  const exit = new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", resolve);
  });
  let stopped = false;
  return {
    samples,
    async stop() {
      if (stopped) return samples;
      stopped = true;
      child.kill();
      const exitCode = await exit;
      if (buffered.trim()) samples.push(JSON.parse(buffered));
      if (exitCode !== null && exitCode !== 0 && samples.length === 0) {
        throw new Error(`Windows memory sampler exited ${exitCode}: ${stderr}`);
      }
      return samples;
    },
  };
}

async function waitForSamples(samples, count, timeoutMs = 10_000) {
  const deadline = Date.now() + timeoutMs;
  while (samples.length < count && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  if (samples.length < count) throw new Error(`Memory sampler produced only ${samples.length} baseline samples.`);
}

function getPosixProcessTree(rootPid) {
  const output = execFileSync("ps", ["-axo", "pid=,ppid=,rss="], { encoding: "utf8" });
  const rows = output.split(/\r?\n/).filter(Boolean).map((line) => {
    const [pid, parentPid, residentKiB] = line.trim().split(/\s+/).map(Number);
    return { pid, parentPid, residentBytes: residentKiB * 1024 };
  });
  const selected = new Set([rootPid]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const row of rows) {
      if (!selected.has(row.pid) && selected.has(row.parentPid)) {
        selected.add(row.pid);
        changed = true;
      }
    }
  }
  return rows.filter((row) => selected.has(row.pid)).map((row) => ({ ...row, privateBytes: null }));
}

async function readBrowserProcessRows(browserName, cdpSession, firefoxRootPid, cachedFirefoxPids) {
  if (browserName === "chromium") {
    const { processInfo } = await cdpSession.send("SystemInfo.getProcessInfo");
    const pids = processInfo.map((item) => item.id);
    if (process.platform === "win32") return getWindowsMemoryForPids(pids);
    return getPosixProcessTree(pids[0]).filter((row) => pids.includes(row.pid));
  }
  if (process.platform === "win32") return getWindowsMemoryForPids(cachedFirefoxPids);
  return getPosixProcessTree(firefoxRootPid);
}

async function profileBrowser(browserName, baseUrl) {
  let browser;
  let browserServer;
  let firefoxRootPid;
  let firefoxPids = [];
  let cdpSession;
  let windowsSampler;
  let server;
  const samples = [];
  let sampling = true;
  let sampleError;

  if (browserName === "chromium") {
    browser = await chromium.launch({ args: ["--enable-webgl", "--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
    cdpSession = await browser.newBrowserCDPSession();
  } else {
    browserServer = await firefox.launchServer();
    firefoxRootPid = browserServer.process().pid;
    browser = await firefox.connect(browserServer.wsEndpoint());
  }

  const sampleLoop = async () => {
    while (sampling) {
      try {
        if (browserName === "firefox" && process.platform === "win32" && samples.length > 0 && samples.length % 6 === 0) {
          firefoxPids = getWindowsFirefoxPids(firefoxRootPid);
        }
        const processes = await readBrowserProcessRows(browserName, cdpSession, firefoxRootPid, firefoxPids);
        if (processes.length) {
          samples.push({
            elapsedMs: Math.round(performance.now()),
            processCount: processes.length,
            residentBytes: processes.reduce((sum, row) => sum + row.residentBytes, 0),
            privateBytes: processes.every((row) => row.privateBytes !== null)
              ? processes.reduce((sum, row) => sum + row.privateBytes, 0)
              : null,
          });
        }
      } catch (error) {
        sampleError = error;
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  };

  try {
    const page = await browser.newPage();
    await page.goto(baseUrl);
    await page.locator(".monaco-editor .view-lines").waitFor({ state: "visible", timeout: 30_000 });
    if (browserName === "firefox" && process.platform === "win32") firefoxPids = getWindowsFirefoxPids(firefoxRootPid);

    let samplingTask;
    if (process.platform === "win32") {
      const pids = browserName === "chromium"
        ? (await cdpSession.send("SystemInfo.getProcessInfo")).processInfo.map((item) => item.id)
        : firefoxPids;
      windowsSampler = startWindowsMemorySampler(pids);
      await waitForSamples(windowsSampler.samples, 2);
    } else {
      samplingTask = sampleLoop();
    }
    const directMeasurement = await page.evaluate(async (source) => {
      const wasm = await import("/wasm/xezim_web.js");
      const exports = await wasm.default();
      const beforeBytes = exports.memory.buffer.byteLength;
      try {
        const json = wasm.compileAndRunProject(JSON.stringify([{ path: "top.sv", contents: source }]), "memory-profile-direct", "top", 20n, 4096, 4096);
        return { beforeBytes, afterBytes: exports.memory.buffer.byteLength, result: JSON.parse(json).status };
      } catch (error) {
        return { beforeBytes, afterBytes: exports.memory.buffer.byteLength, error: String(error) };
      }
    }, ramProject(4_300_000));

    const workerResults = await page.evaluate(async ({ supportedSource, oversizedSource }) => {
      const { runXezimProjectWorker } = await import("/src/workers/xezimClient.ts");
      const run = (source, snapshotId, timeoutMs = 30_000) => runXezimProjectWorker({
        files: [{ path: "top.sv", contents: source }],
        snapshotId,
        topModule: "top",
        maxTimeNs: 20,
        maxOutputBytes: 4096,
        maxVcdBytes: 4096,
      }, { timeoutMs });
      const small = `module top; initial begin $display("RECOVERY=OK"); $finish; end endmodule`;
      return {
        supported: await run(supportedSource, "memory-profile-supported"),
        oversized: await run(oversizedSource, "memory-profile-over-limit", 120_000),
        recovery: await run(small, "memory-profile-recovery"),
      };
    }, { supportedSource: ramProject(1_048_576), oversizedSource: ramProject(4_300_000) });

    await page.waitForTimeout(300);
    sampling = false;
    if (windowsSampler) samples.push(...await windowsSampler.stop());
    else await samplingTask;
    if (sampleError) throw sampleError;
    if (!directMeasurement.error?.includes("RuntimeError")) throw new Error(`Expected direct WASM allocation trap; got ${JSON.stringify(directMeasurement)}`);
    if (!workerResults.supported.ok || !workerResults.supported.result.stdout.includes("RAM0=1")) throw new Error("Supported Worker workload failed.");
    if (workerResults.oversized.ok || workerResults.oversized.error.code !== "RESOURCE_LIMIT") throw new Error("Over-limit Worker workload was not reported as RESOURCE_LIMIT.");
    if (!workerResults.recovery.ok || !workerResults.recovery.result.stdout.includes("RECOVERY=OK")) throw new Error("Worker did not recover after the resource limit.");
    if (samples.length === 0) throw new Error("No browser process memory samples were collected.");

    const peakResident = samples.reduce((peak, sample) => Math.max(peak, sample.residentBytes), 0);
    const privateSamples = samples.filter((sample) => sample.privateBytes !== null);
    const peakPrivate = privateSamples.length ? Math.max(...privateSamples.map((sample) => sample.privateBytes)) : null;
    return {
      browser: browserName,
      peakAggregateResidentMiB: +(peakResident / 1024 ** 2).toFixed(1),
      peakAggregatePrivateMiB: peakPrivate === null ? null : +(peakPrivate / 1024 ** 2).toFixed(1),
      samples: samples.length,
      processCountRange: [Math.min(...samples.map((sample) => sample.processCount)), Math.max(...samples.map((sample) => sample.processCount))],
      wasm: directMeasurement,
      worker: {
        supported: workerResults.supported.result.status,
        overLimit: workerResults.oversized.error.code,
        recovery: workerResults.recovery.result.status,
      },
    };
  } finally {
    sampling = false;
    if (windowsSampler) await windowsSampler.stop().catch(() => undefined);
    await browser?.close().catch(() => undefined);
    await browserServer?.close().catch(() => undefined);
    await server?.close().catch(() => undefined);
  }
}

const wasmGlue = path.join(appRoot, "public", "wasm", "xezim_web.js");
await readFile(wasmGlue);
const viteServer = await createServer({
  configFile: path.join(appRoot, "vite.config.ts"),
  root: appRoot,
  logLevel: "error",
  server: { host: "127.0.0.1", port: 0, strictPort: false },
});
await viteServer.listen();
const address = viteServer.httpServer.address();
const baseUrl = `http://127.0.0.1:${address.port}`;

try {
  const results = [];
  for (const browserName of browserNames) results.push(await profileBrowser(browserName, baseUrl));
  console.log(JSON.stringify({
    host: {
      platform: os.platform(),
      release: os.release(),
      totalRamGiB: +(os.totalmem() / 1024 ** 3).toFixed(1),
      logicalCpuCount: os.cpus().length,
      cpuModel: os.cpus()[0]?.model?.trim(),
    },
    note: "Aggregate process resident sets can count shared pages more than once; private bytes are reported where the OS exposes them.",
    results,
  }, null, 2));
} finally {
  await viteServer.close();
}
