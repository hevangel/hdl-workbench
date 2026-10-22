import { test, expect } from "@playwright/test";

const ramProject = (wordCount) => `module top;
  logic [31:0] ram [0:${wordCount - 1}];
  initial begin
    ram[0] = 32'd1;
    $display("RAM0=%0d", ram[0]);
    $finish;
  end
endmodule
`;

test("WASM simulation stays within the memory ceiling and reports exhaustion", async ({ page }) => {
  await page.goto("/");
  const directMeasurement = await page.evaluate(async (source) => {
    const wasm = await import("/wasm/xezim_web.js");
    const exports = await wasm.default();
    const beforeBytes = exports.memory.buffer.byteLength;
    try {
      const json = wasm.compileAndRunProject(JSON.stringify([{ path: "top.sv", contents: source }]), "luna10-measurement", "top", 20n, 4096, 4096);
      return { beforeBytes, afterBytes: exports.memory.buffer.byteLength, result: JSON.parse(json).status };
    } catch (error) {
      return { beforeBytes, afterBytes: exports.memory.buffer.byteLength, error: String(error) };
    }
  }, ramProject(4_300_000));
  expect(directMeasurement.afterBytes).toBeLessThanOrEqual(256 * 1024 * 1024);
  expect(directMeasurement.afterBytes).toBeGreaterThanOrEqual(224 * 1024 * 1024);
  expect(directMeasurement.error).toContain("RuntimeError");
  const responses = await page.evaluate(async ({ supportedSource, oversizedSource }) => {
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
      supported: await run(supportedSource, "luna10-supported"),
      oversized: await run(oversizedSource, "luna10-over-limit", 120_000),
      recovery: await run(small, "luna10-recovery"),
    };
  }, {
    supportedSource: ramProject(1_048_576),
    oversizedSource: ramProject(4_300_000),
  });

  expect(responses.supported.ok).toBe(true);
  expect(responses.supported.result.status).toBe("completed");
  expect(responses.supported.result.stdout).toContain("RAM0=1");
  expect(responses.oversized).toMatchObject({
    ok: false,
    error: { code: "RESOURCE_LIMIT" },
  });
  expect(responses.oversized.error.message).toContain("256 MiB");
  expect(responses.recovery.ok).toBe(true);
  expect(responses.recovery.result.stdout).toContain("RECOVERY=OK");
});
