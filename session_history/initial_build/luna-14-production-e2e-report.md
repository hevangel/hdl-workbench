# LUNA-14 — Production bundle browser validation

Date: 2026-10-22
Branch: `codex/luna-14-production-e2e`

## Result

The production Vite bundle passed the full six-case E2E suite in Chrome and
Firefox. The six cases cover MCP edit/run, ROM and NBA behavior, Surfer VCD and
reload, WASM memory exhaustion and recovery, and C/C++ compilation followed by
real Xezim DPI calls.

The memory test initially failed against `vite preview` because it imported
`/src/workers/xezimClient.ts`, a development-only URL. It now uses a dedicated
`memory-harness.html` entry that Vite bundles for both development and
production. The production test then passed in full.

## Build and reproduce

Build the application and all workspace packages:

```powershell
pnpm build
```

Result: pass. The build emits the IDE, worker harness, memory harness, Xezim
WASM, Surfer files, compiler assets, and license notices. Vite reports the
existing application-chunk size warning; the compiler asset sizes and
first-use memory profile are recorded in
[`luna-12-dpi-memory-report.md`](luna-12-dpi-memory-report.md).

In one terminal, serve the built `apps/ide/dist` directory:

```powershell
pnpm --filter @xezim-ide/app exec vite preview --host 127.0.0.1 --port 4173 --strictPort
```

In another terminal, run the suite against the production preview:

```powershell
$env:XEZIM_PLAYWRIGHT_BASE_URL = 'http://127.0.0.1:4173'
$env:XEZIM_CHROME_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
pnpm --filter @xezim-ide/mcp-bridge test:e2e
```

Result: 6 passed in 35.9 seconds across Chrome `156.0.8078.47` and the
Playwright Firefox project. Firefox's exact bundled browser build was not
captured. The same six cases also pass against Vite's development server in
41.6 seconds; see [`luna-13-dpi-cross-browser-report.md`](luna-13-dpi-cross-browser-report.md).

## Scope

This confirms same-origin production asset paths and worker loading for the
local production bundle. It does not test a public CDN or hosted Site, public
network transfer time, or Firefox process memory. Source publication, a
canonical hosting origin, and qualified license review remain separate release
gates.
