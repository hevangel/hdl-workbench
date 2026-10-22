import { test, expect } from "@playwright/test";

test("browser C and C++ compile to modules that Xezim calls through DPI", async ({ page }) => {
  await page.goto("/worker-harness.html");
  const result = page.locator("#result");
  await expect(result).toHaveAttribute("data-status", "passed", { timeout: 180_000 });

  const output = JSON.parse((await result.textContent()) ?? "{}");
  expect(output.passed).toBe(true);
  expect(output.invalidDpi).toMatchObject({
    ok: false,
    error: { code: "DPI_COMPILATION_FAILED" },
  });
  expect(output.compiledDpi).toMatchObject({
    ok: true,
    modules: ["dpi/add.c", "dpi/add_cpp.cpp"],
  });
  expect(output.directDpi.map(({ value }) => value)).toEqual([30, 42]);
  expect(output.dpiResult).toMatchObject({
    status: "completed",
    finished: true,
  });
  expect(output.dpiResult.stdout).toContain("DPI_C=30");
  expect(output.dpiResult.stdout).toContain("DPI_CPP=42");
});
