import { test, expect } from "@playwright/test";

test("WASM simulation stays within the memory ceiling and reports exhaustion", async ({ page }) => {
  await page.goto("/memory-harness.html");
  const result = page.locator("#result");
  await expect(result).toHaveAttribute("data-status", "passed", { timeout: 180_000 });
  const output = JSON.parse((await result.textContent()) ?? "{}");
  expect(output.passed).toBe(true);
  expect(output.direct.afterBytes).toBeLessThanOrEqual(256 * 1024 * 1024);
  expect(output.direct.afterBytes).toBeGreaterThanOrEqual(224 * 1024 * 1024);
  expect(output.direct.error).toContain("RuntimeError");
  expect(output.supported).toMatchObject({ status: "completed" });
  expect(output.supported.stdout).toContain("RAM0=1");
  expect(output.overLimit.code).toBe("RESOURCE_LIMIT");
  expect(output.overLimit.message).toContain("256 MiB");
  expect(output.recovery.stdout).toContain("RECOVERY=OK");
});
