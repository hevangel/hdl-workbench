import { runXezimProjectWorker } from "./xezimClient";

const ramProject = (wordCount: number) => `module top;
  logic [31:0] ram [0:${wordCount - 1}];
  initial begin
    ram[0] = 32'd1;
    $display("RAM0=%0d", ram[0]);
    $finish;
  end
endmodule
`;

interface XezimModule {
  default: () => Promise<{ memory: WebAssembly.Memory }>;
  compileAndRunProject: (
    filesJson: string,
    snapshotId: string,
    topModule: string,
    maxTimeNs: bigint,
    maxOutputBytes: number,
    maxVcdBytes: number,
  ) => string;
}

async function main(): Promise<void> {
  const output = document.querySelector<HTMLPreElement>("#result");
  if (!output) throw new Error("Missing #result memory acceptance output element.");
  try {
    const wasmModuleUrl = new URL("/wasm/xezim_web.js", window.location.href);
    const wasm = await import(/* @vite-ignore */ wasmModuleUrl.href) as XezimModule;
    const exports = await wasm.default();
    const beforeBytes = exports.memory.buffer.byteLength;
    let directError = "";
    let directBytes = exports.memory.buffer.byteLength;
    try {
      wasm.compileAndRunProject(
        JSON.stringify([{ path: "top.sv", contents: ramProject(4_300_000) }]),
        "memory-harness-direct",
        "top",
        20n,
        4096,
        4096,
      );
    } catch (error) {
      directError = String(error);
      directBytes = exports.memory.buffer.byteLength;
    }

    const run = (source: string, snapshotId: string, timeoutMs = 30_000) => runXezimProjectWorker({
      files: [{ path: "top.sv", contents: source }],
      snapshotId,
      topModule: "top",
      maxTimeNs: 20,
      maxOutputBytes: 4096,
      maxVcdBytes: 4096,
    }, { timeoutMs });
    const small = `module top; initial begin $display("RECOVERY=OK"); $finish; end endmodule`;
    const supported = await run(ramProject(1_048_576), "memory-harness-supported");
    const oversized = await run(ramProject(4_300_000), "memory-harness-over-limit", 120_000);
    const recovery = await run(small, "memory-harness-recovery");
    const passed = directError.includes("RuntimeError")
      && directBytes >= 224 * 1024 * 1024
      && directBytes <= 256 * 1024 * 1024
      && supported.ok
      && supported.result.status === "completed"
      && supported.result.stdout.includes("RAM0=1")
      && !oversized.ok
      && oversized.error.code === "RESOURCE_LIMIT"
      && oversized.error.message.includes("256 MiB")
      && recovery.ok
      && recovery.result.stdout.includes("RECOVERY=OK");
    output.dataset.status = passed ? "passed" : "failed";
    output.textContent = JSON.stringify({
      passed,
      direct: { beforeBytes, afterBytes: directBytes, error: directError },
      supported: supported.ok
        ? { status: supported.result.status, stdout: supported.result.stdout }
        : supported,
      overLimit: oversized.ok ? oversized : oversized.error,
      recovery: recovery.ok
        ? { status: recovery.result.status, stdout: recovery.result.stdout }
        : recovery,
    }, null, 2);
  } catch (error) {
    output.dataset.status = "failed";
    output.textContent = error instanceof Error ? error.stack ?? error.message : String(error);
  }
}

void main();
