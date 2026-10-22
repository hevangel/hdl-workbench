# LUNA-09 — Browser integration and acceptance

Date: 2026-10-22
Branch: `codex/luna-09-integration`
Status: the local Chromium end-to-end path passes against the generated Xezim WASM and pinned Surfer viewer. The project now has a browser-controller path shared by the human Run button and authenticated MCP execution.

## Integration behavior

- `BrowserIdeController.execute()` is the common one-shot operation used by the Run button and MCP `simulation.execute` RPC. It freezes project files and revisions, checks snapshot provenance, runs the pinned Xezim project Worker, stores complete VCD bytes in OPFS, and returns result and artifact metadata.
- The upstream browser Xezim API exposes combined compile-and-run only. Separate compile/run requests return `UNSUPPORTED_CAPABILITY`; MCP `run_simulation` uses the explicit `simulation.execute` operation. See [ADR-004](ADR-004-one-shot-browser-execution.md).
- When authenticated MCP operations change the browser workspace, the controller notifies the IDE. The app refreshes project files and revisions and updates the editor without requiring a page reload.
- The local MCP client accepts loopback `ws://` URLs only, pairs using the bridge challenge, checks protocol messages and forwards RPCs to the controller. The connected project editor is read-only so the local project remains the source of truth.
- The browser build reports simulation unavailable if the generated WASM module or binary is absent. No native Xezim fallback is configured.

## Reproduction

Use Rust `1.99.1`, target `wasm32-unknown-unknown`, clang, wasm-bindgen CLI `0.2.129`, Node 24, and pnpm `11.25.0`. From a clean checkout:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
pnpm install --frozen-lockfile
cargo build --release --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown
mkdir -p apps/ide/public/wasm
wasm-bindgen --target web --out-dir apps/ide/public/wasm \
  crates/xezim-web/target/wasm32-unknown-unknown/release/xezim_web.wasm
pnpm typecheck
pnpm build
pnpm test
pnpm --filter @xezim-ide/mcp-bridge exec playwright install --with-deps chromium
pnpm test:e2e
```

The generated `apps/ide/public/wasm/` files are ignored; a clean checkout must regenerate them before serving the app. The end-to-end test starts the IDE at `http://127.0.0.1:5174`, creates a temporary project root and local MCP bridge, pairs the browser, edits the counter through MCP, runs Xezim WASM, verifies the returned `COUNT=4` and complete OPFS VCD, opens it in Surfer, reloads, and opens the persisted artifact again.

CI runs the same browser acceptance on Ubuntu with the pinned Rust toolchain and Chromium. Playwright's CI instructions use browser installation with system dependencies and a single worker for stability; this repository's config runs one worker and installs Chromium only. See [Playwright CI](https://playwright.dev/docs/ci) and [Playwright browsers](https://playwright.dev/docs/browsers).

## Verification record

| Check | Result |
| --- | --- |
| `pnpm install --no-frozen-lockfile` | Passed; added pinned `@playwright/test` and updated `pnpm-lock.yaml`. |
| `pnpm typecheck` | Passed across protocol, IDE, bridge and wave-index packages. |
| `pnpm build` | Passed; Vite still reports the known large Monaco application chunk. |
| `pnpm test` | Passed: protocol 20, IDE 28, MCP bridge 6, wave-index 6. MCP WebSocket integration needs local loopback access. |
| `pnpm test:e2e` | Passed in Chromium `156.0.8078.4`; 1 test. It exercised MCP source edit, real browser Worker simulation, VCD output, Surfer load, OPFS reload, and re-pairing. |
| `git diff --check` | Passed after final edits. |

The browser test uses the Xezim counter fixture with `$dumpfile` and `$dumpvars` inserted into its test copy. The original minimal counter fixture emits no VCD timescale; the controller correctly refuses to label such a run as a waveform instead of inventing metadata.

## Unresolved boundaries

- The browser Xezim entrypoint remains combined compile-and-run. Separate compile handles, instruction-step budgeting, and mid-run cancellation are not implemented and are not advertised.
- Worker execution uses the existing integer-nanosecond stop-time interface and its 10,000,000 ns bound. Fractional nanosecond requests are rejected.
- Output limits, VCD limits, and a Worker timeout are enforced by existing Xezim/browser code, but there is no hard WASM linear-memory ceiling.
- This check used Chromium only. A second browser engine and a manual real Codex MCP smoke test remain unverified.
- Native Xezim was not modified by LUNA-09. Native suite passes are recorded in the LUNA-03/LUNA-04 reports; rerun those suites in the Linux builder before release.

## Recommended next work

LUNA-10 should exercise the CI workflow from a clean checkout, run the full native Xezim compatibility suite, test another browser engine if available, review workspace authority and quota/memory bounds, and complete licensing, release and unsupported-capability documentation. Keep the combined browser execution limit explicit until the pinned Xezim API gains separate compile/run and cancellation support.
