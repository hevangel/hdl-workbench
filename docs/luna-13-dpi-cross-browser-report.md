# LUNA-13 — Cross-browser DPI compiler verification

Date: 2026-10-22
Branch: `codex/luna-13-cross-browser-dpi`

## Result

The browser C/C++ compiler and the actual Xezim WASM DPI call passed in both
configured Playwright browser projects. The regression uses the real
`apps/ide/worker-harness.html` page and its existing worker acceptance flow; it
does not mock Clang, LLD, DPI modules, or the simulator.

The harness verifies the C syntax-error result, then compiles one C and one
C++20 translation unit with project headers. It invokes the generated modules
directly (results 30 and 42), passes them into the Xezim browser Worker, and
checks the completed SystemVerilog output for `DPI_C=30` and `DPI_CPP=42`.
The harness also runs the counter, multi-file/ROM/VCD, invalid SystemVerilog,
and simulator-worker recovery cases.

## Browsers and commands

- Chrome `156.0.8078.47`, using the installed system executable.
- Firefox using the configured Playwright `1.64.0` browser project. The exact
  bundled Firefox build was not captured in this run.

Targeted DPI test:

```powershell
$env:XEZIM_CHROME_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
pnpm --filter @xezim-ide/mcp-bridge exec playwright test e2e/luna11-dpi.spec.mjs --project=chromium --project=firefox
```

Result: 2 passed in 18.9 seconds.

Full browser regression:

```powershell
$env:XEZIM_CHROME_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
pnpm --filter @xezim-ide/mcp-bridge test:e2e
```

Result: 6 passed in 40.9 seconds across Chrome and Firefox against the Vite
development server. The command builds
the MCP bridge, starts the local Vite server, and runs the MCP/Surfer reload,
memory ceiling/recovery, and DPI compiler scenarios in each browser.

## Scope and remaining limits

This establishes functional cross-browser compilation and DPI integration for
the supported scalar `int` ABI. It does not measure Firefox process memory or
public-network first-use download time. The memory measurements remain the
Chrome-only selected-workstation results in
[`luna-12-dpi-memory-report.md`](luna-12-dpi-memory-report.md). Unsupported DPI
types and the browser C/C++ restrictions remain as documented in
[`luna-11-dpi-c-cpp-report.md`](luna-11-dpi-c-cpp-report.md).
