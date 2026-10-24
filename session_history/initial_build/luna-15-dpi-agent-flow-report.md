# LUNA-15 — MCP project C/C++ DPI flow

Date: 2026-10-22
Branch: `codex/luna-15-dpi-agent-flow`

## Result

The real MCP-to-browser project path now compiles and runs C and C++ DPI code,
reports compiler failures, and recovers after the source is repaired. The test
starts the MCP stdio bridge on a temporary project, pairs it with the browser,
and checks the actual simulator output: `DPI_C=30` and `DPI_CPP=42`.

The test exposed a snapshot ordering defect. The bridge hashed files in recursive
directory traversal order, while the browser hashed a flat locale-sorted list.
For a project containing root-level `dpi_top.sv` and nested `dpi/` files, the
orders differed and a synchronized, byte-identical project was rejected with
`REVISION_CONFLICT`. Snapshot ordering now uses a shared UTF-16 code-unit path
comparator in the protocol package, MCP bridge, OPFS workspace, and browser
snapshot hash. This gives Node and browser engines the same stable order without
relying on the current locale.

The DPI fixture declares its timescale and starts VCD capture explicitly. This
ensures the simulation returns supported time metadata through the same result
path expected from ordinary waveform-producing projects.

## Changed files and interface

- Added `compareProjectPaths(left, right)` to the shared protocol package. It
  compares normalized path strings by JavaScript UTF-16 code units and is used
  for MCP file listings, OPFS file listings and snapshots, and bridge snapshot
  hashing.
- Added `services/mcp-bridge/e2e/luna15-dpi-agent-flow.spec.mjs` to exercise the
  real stdio MCP bridge, disk project synchronization, C/C++ DPI execution,
  compiler error reporting, and recovery in both browsers.
- Updated the DPI fixture with explicit timescale and VCD setup; updated this
  report and the release readiness matrix.
- No Xezim or xezim-core source, build flags, or native APIs changed.

## Reproduce

Run from the project root with Node 24+, pnpm 11.25.0, and the installed
Playwright Chromium and Firefox browsers:

```powershell
pnpm --filter @xezim-ide/protocol test
pnpm typecheck
pnpm build
$env:XEZIM_CHROME_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
pnpm test:e2e
```

Results on 2026-10-22:

- Protocol tests: 21 passed.
- Workspace typecheck: passed.
- Production build: passed; Vite reported the existing large application
  chunk warning.
- Development browser suite: 8 passed across Chromium and Firefox in 59.7 s.
- Complete unit suite (`pnpm test`): 72 passed (protocol 21, IDE 38, bridge 7,
  wave-index 6). The bridge integration tests bind ephemeral loopback ports.

To verify the production bundle, build first, start preview in one terminal,
then run Playwright from another:

```powershell
pnpm --filter @xezim-ide/app exec vite preview --host 127.0.0.1 --port 4173 --strictPort
```

```powershell
$env:XEZIM_PLAYWRIGHT_BASE_URL = 'http://127.0.0.1:4173'
$env:XEZIM_CHROME_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
pnpm --filter @xezim-ide/mcp-bridge test:e2e
```

Result: 8 passed across Chromium and Firefox against the production Vite preview
in 57.7 s. The exact Firefox browser version was not captured for this run.

The focused case can be run with:

```powershell
pnpm --filter @xezim-ide/mcp-bridge exec playwright test --config=playwright.config.mjs e2e/luna15-dpi-agent-flow.spec.mjs --project=chromium --project=firefox
```

The case verifies this sequence:

1. Pair the browser with the local MCP bridge.
2. Run C and C++ functions from the selected disk project through browser Xezim
   WASM and verify their exact output.
3. Write invalid C source with an expected revision and verify the run reports
   `DPI_COMPILATION_FAILED`.
4. Restore valid source using the new revision and verify both DPI calls succeed.

## Limits

This validates local development and production bundles in the tested Chromium
and Playwright Firefox projects. It does not establish a public hosted-site
origin, public network performance, minimum hardware requirements, or legal
approval. The source branch is published in [PR #1](https://github.com/hevangel/hdl-workbench/pull/1);
the hosted-site and qualified-review gates remain open. See [v1 release readiness](v1-release-readiness.md).
