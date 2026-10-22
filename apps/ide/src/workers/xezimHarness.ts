import { runXezimProjectWorker, runXezimWorker } from "./xezimClient";
import { compileDpiProjectInBrowser } from "./dpiCompilerClient";

function countTransitions(vcd: string): Array<{ time: string; value: string }> {
  const declaration = vcd.split("\n").find((line) => /\$var\s+\w+\s+4\s+\S+\s+count\s+\[3:0\]/.test(line));
  if (!declaration) return [];
  const code = declaration.match(/\$var\s+\w+\s+4\s+(\S+)\s+count\s+\[3:0\]/)?.[1];
  if (!code) return [];
  const result: Array<{ time: string; value: string }> = [];
  let time = "0";
  for (const line of vcd.split("\n")) {
    if (line.startsWith("#")) {
      time = line.slice(1);
      continue;
    }
    const value = line.match(/^b([01xzXZ]+)\s+(\S+)$/);
    if (value?.[2] === code) result.push({ time, value: value[1]!.toLowerCase() });
  }
  return result.filter((transition) => !/[xz]/.test(transition.value));
}

async function loadProjectFiles(): Promise<Array<{ path: string; contents: string }>> {
  const paths = [
    "rtl/marker_pkg.sv",
    "rtl/top.sv",
    "rtl/includes/top_defs.svh",
    "rtl/includes/nested.svh",
    "rom.hex",
  ];
  return Promise.all(paths.map(async (path) => {
    const response = await fetch(`/fixtures/project/${path}`);
    if (!response.ok) throw new Error(`Could not load project fixture ${path} (${response.status}).`);
    return { path, contents: await response.text() };
  }));
}

async function loadTextFixture(path: string): Promise<string> {
  const response = await fetch(`/fixtures/${path}`);
  if (!response.ok) throw new Error(`Could not load fixture ${path} (${response.status}).`);
  return response.text();
}

async function main(): Promise<void> {
  const output = document.querySelector<HTMLPreElement>("#result");
  if (!output) throw new Error("Missing #result acceptance output element.");
  try {
    const fixture = await fetch("/fixtures/counter.sv");
    if (!fixture.ok) throw new Error(`Could not load counter fixture (${fixture.status}).`);
    const counterInput = {
      source: await fixture.text(),
      topModule: "counter",
      maxTimeNs: 50,
    };
    const response = await runXezimWorker(counterInput);
    const projectResponse = await runXezimProjectWorker({
      files: await loadProjectFiles(),
      snapshotId: "sha256:luna-04-browser-fixture",
      topModule: "top",
      maxTimeNs: 20,
    });
    const invalidSource = await runXezimWorker({
      source: "module invalid syntax; endmodule",
      topModule: "invalid",
      maxTimeNs: 50,
    });
    const recovery = await runXezimWorker(counterInput);
    const dpiC = await loadTextFixture("dpi/add.c");
    const dpiCpp = await loadTextFixture("dpi/add_cpp.cpp");
    const dpiHeader = await loadTextFixture("dpi/add.h");
    const dpiCppHeader = await loadTextFixture("dpi/add_cpp.hpp");
    const dpiSv = await loadTextFixture("dpi/top.sv");
    const invalidDpi = await compileDpiProjectInBrowser([{ path: "dpi/broken.c", contents: "int not_closed( {" }]);
    const compiledDpi = await compileDpiProjectInBrowser([
      { path: "dpi/add.c", contents: dpiC },
      { path: "dpi/add_cpp.cpp", contents: dpiCpp },
      { path: "dpi/add.h", contents: dpiHeader },
      { path: "dpi/add_cpp.hpp", contents: dpiCppHeader },
    ]);
    const directDpi = compiledDpi.ok
      ? await Promise.all(compiledDpi.modules.map(async ({ path, bytes }) => {
        const instance = await WebAssembly.instantiate(bytes, {});
        const module = await WebAssembly.compile(bytes);
        const symbol = path.endsWith(".cpp") ? "add_ints_cpp" : "add_ints";
        const exported = instance.instance.exports[symbol];
        const args = path.endsWith(".cpp") ? [19, 23] : [10, 20];
        return {
          path,
          exportNames: WebAssembly.Module.exports(module).filter(({ kind }) => kind === "function").map(({ name }) => name),
          value: typeof exported === "function" ? (exported as (left: number, right: number) => number)(args[0]!, args[1]!) : null,
        };
      }))
      : [];
    const dpiResult = compiledDpi.ok
      ? await runXezimWorker({
        source: dpiSv,
        topModule: "dpi_top",
        maxTimeNs: 1,
        dpiModules: compiledDpi.modules.map(({ path, bytes }) => ({ path: `${path}.wasm`, bytes })),
      })
      : null;
    const passed = response.ok
      && response.result.status === "completed"
      && response.result.finished
      && response.result.stdout.includes("COUNT=4")
      && projectResponse.ok
      && projectResponse.result.status === "completed"
      && projectResponse.result.snapshotId === "sha256:luna-04-browser-fixture"
      && projectResponse.result.waveform?.complete === true
      && projectResponse.result.waveform.format === "vcd"
      && projectResponse.result.waveform.timescale === "1ps"
      && projectResponse.result.vcd?.startsWith("$date") === true
      && projectResponse.result.vcd.includes("10xz")
      && JSON.stringify(countTransitions(projectResponse.result.vcd)) === JSON.stringify([
        { time: "0", value: "1" },
        { time: "1000", value: "10" },
        { time: "3000", value: "11" },
      ])
      && projectResponse.result.stdout.includes("FILE=ROM=f0")
      && projectResponse.result.stdout.includes("NBA_PRE=1")
      && projectResponse.result.stdout.includes("NBA_POST=2")
      && projectResponse.result.stdout.includes("ROM=f0 COUNT=3")
      && projectResponse.result.fileOutputs?.some((file) => file.path === "captured.txt" && file.contents === "ROM=f0\n") === true
      && invalidSource.ok
      && invalidSource.result.status === "failed"
      && invalidSource.result.diagnostics.some((diagnostic) => diagnostic.severity === "error")
      && recovery.ok
      && recovery.result.status === "completed"
      && recovery.result.finished
      && recovery.result.stdout.includes("COUNT=4")
      && !invalidDpi.ok
      && invalidDpi.error.code === "DPI_COMPILATION_FAILED"
      && compiledDpi.ok
      && compiledDpi.modules.length === 2
      && directDpi[0]?.value === 30
      && directDpi[1]?.value === 42
      && dpiResult?.ok === true
      && dpiResult.result.status === "completed"
      && dpiResult.result.finished
      && dpiResult.result.stdout.includes("DPI_C=30")
      && dpiResult.result.stdout.includes("DPI_CPP=42");
    output.dataset.status = passed ? "passed" : "failed";
    output.textContent = JSON.stringify({
      passed,
      counter: response.ok ? { status: response.result.status, finished: response.result.finished, stdout: response.result.stdout } : response,
      project: projectResponse.ok ? {
        status: projectResponse.result.status,
        waveform: projectResponse.result.waveform,
        transitionCount: projectResponse.result.vcd ? countTransitions(projectResponse.result.vcd).length : 0,
        stdout: projectResponse.result.stdout,
      } : projectResponse,
      invalidSource: invalidSource.ok ? { status: invalidSource.result.status, errorCount: invalidSource.result.diagnostics.filter(({ severity }) => severity === "error").length } : invalidSource,
      recovery: recovery.ok ? { status: recovery.result.status, finished: recovery.result.finished, stdout: recovery.result.stdout } : recovery,
      invalidDpi,
      compiledDpi: compiledDpi.ok ? { ok: true, modules: compiledDpi.modules.map(({ path }) => path), output: compiledDpi.output } : compiledDpi,
      directDpi,
      dpiResult: dpiResult?.ok === true ? { status: dpiResult.result.status, finished: dpiResult.result.finished, stdout: dpiResult.result.stdout } : dpiResult,
    }, null, 2);
  } catch (error) {
    output.dataset.status = "failed";
    output.textContent = error instanceof Error ? error.stack ?? error.message : String(error);
  }
}

void main();
