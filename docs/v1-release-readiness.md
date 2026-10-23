# v1 release readiness

Date: 2026-10-22
Branch: `codex/luna-15-dpi-agent-flow`
Status: feature, development and production browser integration, selected-workstation memory checks, and source-branch publication pass; PR review, hosted-origin checks, and qualified release review remain open.

## Pins

| Component | Pin |
| --- | --- |
| Xezim | `22faa127a9cc0156bd23e0ab117f400c64231913` (`0.11.2`) |
| xezim-core and parser | `d15cdc39e27a4c0a8a55f7e1d46369181307ead9` (`0.11.2`) |
| Surfer | `bd749b1f786c1c62cd67893ca71346cbe6983915` (`v0.7.0`) |
| Rust/Cargo | `1.99.1` |
| wasm-bindgen CLI | `0.2.129` |
| Node / pnpm | Node `24` or newer in the supported range / pnpm `11.25.0` |

Source pins and original licenses for Xezim are in [`vendor/UPSTREAM.md`](../vendor/UPSTREAM.md). Surfer's upstream asset hashes and retained EUPL-1.2 notice are in [`apps/ide/public/surfer/UPSTREAM.md`](../apps/ide/public/surfer/UPSTREAM.md) and `UPSTREAM-SHA256SUMS`.

## Reproducible build and demo

Run from a clean checkout with the toolchain above and clang installed:

```sh
pnpm install --frozen-lockfile
cargo install wasm-bindgen-cli --version 0.2.129 --locked
pnpm build:browser
pnpm build
pnpm demo
```

`pnpm build:browser` runs `rustup target add wasm32-unknown-unknown`, builds the pinned `crates/xezim-web` release artifact with Cargo's lockfile, then runs the pinned wasm-bindgen CLI into ignored `apps/ide/public/wasm/`. `pnpm demo` rebuilds WASM and starts the local IDE. If generation fails, browser simulation stays unavailable; the application never switches to native simulation.

## Acceptance matrix

| Check | Result | Evidence / command |
| --- | --- | --- |
| Xezim WASM build and browser Worker | Pass on LUNA-03 | `docs/luna-03-wasm-report.md`; debug and release builds plus real Chromium counter and failure recovery. |
| Multi-file ROM, `$readmemh`, `$fopen`, NBA and four-state fixture | Pass on LUNA-04 and current end-to-end | `apps/ide/src/workers/xezimHarness.ts`; current `pnpm test:e2e` checks the ROM and NBA output through MCP in both browsers. |
| Surfer fixture VCD and integration messages | Pass on LUNA-00B/LUNA-05 | `docs/surfer-integration-report.md`; pinned browser API and real VCD fixture. |
| Surfer license/source notice and served assets | Pass in Chromium and Firefox | `pnpm --filter @xezim-ide/mcp-bridge exec playwright test e2e/luna09.spec.mjs --project=chromium --project=firefox`; verifies dated attribution, EUPL text, pinned upstream link, and downloadable local wrapper source. |
| End-to-end MCP edit → Xezim WASM → OPFS VCD → Surfer → reload | Pass in Chrome `156.0.8078.47` and the Playwright Firefox project | `pnpm --filter @xezim-ide/mcp-bridge test:e2e`; ROM plus counter run. |
| Frozen workspace install | Pass with pnpm `11.25.0`; all five workspace projects already up to date | `pnpm install --frozen-lockfile`. |
| `pnpm typecheck` | Pass | All four workspace packages. |
| `pnpm build` | Pass; existing Monaco bundle size warning | All workspace packages and generated protocol JSON schema. |
| Production bundle E2E | Pass; all 8 browser cases across Chromium and Firefox in 57.7 seconds, including real MCP project C/C++ DPI compilation and recovery | `pnpm build`, serve `apps/ide/dist` with `vite preview`, then `pnpm --filter @xezim-ide/mcp-bridge test:e2e` with `XEZIM_PLAYWRIGHT_BASE_URL`; see `docs/luna-15-dpi-agent-flow-report.md` and historical `docs/luna-14-production-e2e-report.md`. |
| `pnpm test` | Pass with local loopback permission; 72 tests | Protocol 21, IDE 38, MCP bridge 7, wave-index 6. |
| Capped `pnpm build:browser` equivalent | Pass in the pinned Linux builder | `cargo build --locked --release --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown`, then `wasm-bindgen --target web --out-dir apps/ide/public/wasm crates/xezim-web/target/wasm32-unknown-unknown/release/xezim_web.wasm`; see `docs/luna-10-memory-report.md`. |
| Concurrent same-revision bridge writes | Covered by regression test | `pnpm test`; exactly one concurrent write succeeds. |
| Source project and waveform bounds | Implemented with regression coverage | `pnpm test`; see limits below. |
| Simulator console capture bound | Pass; captured during simulation, Unicode-safe, coalesced, and reported as a resource limit | `cargo test --manifest-path crates/xezim-web/Cargo.toml`; native `simulate_project_inline` keeps its unbounded default. |
| LUNA-13 full development E2E in Chromium + Firefox | Pass; 6 cases across 2 browsers in 40.9 seconds | `pnpm --filter @xezim-ide/mcp-bridge test:e2e`; MCP edit/run, ROM/NBA, Surfer waveform/reload, memory ceiling/recovery, and C/C++ DPI compilation. See `docs/luna-13-dpi-cross-browser-report.md`. |
| Browser C/C++ compile and SystemVerilog DPI | Pass in Chrome `156.0.8078.47` and the Playwright Firefox project; C returns 30 and C++ returns 42 | `pnpm --filter @xezim-ide/mcp-bridge exec playwright test e2e/luna11-dpi.spec.mjs --project=chromium --project=firefox`; see `docs/luna-11-dpi-c-cpp-report.md` and `docs/luna-13-dpi-cross-browser-report.md`. |
| MCP disk project → browser C/C++ DPI → compiler error → recovery | Pass in Chromium and Firefox in development and production bundle; exact C/C++ outputs verified | `pnpm test:e2e`; see [`luna-15-dpi-agent-flow-report.md`](luna-15-dpi-agent-flow-report.md). |
| Compiler-inclusive whole-browser process memory | Pass on selected Windows workstation with Chrome `156.0.8078.47`; during compile/DPI, peak sampled resident 1,160.7 MiB and private 1,426.9 MiB | `pnpm --filter @xezim-ide/mcp-bridge profile:browser-memory -- chromium` with `XEZIM_CHROME_EXECUTABLE`; see `docs/luna-12-dpi-memory-report.md`. This does not establish a lower-resource minimum or Firefox compiler memory. |
| WASM memory ceiling and failure recovery | Pass in Chromium `156.0.8078.4` and Firefox `157.0` | `pnpm --filter @xezim-ide/mcp-bridge exec playwright test e2e/luna10-memory.spec.mjs --project=chromium` and `--project=firefox`; see `docs/luna-10-memory-report.md`. |
| Whole-browser process memory on selected v1 baseline | Pass on the user-selected computer: Windows 10.0.26300, Ryzen 9 5950X, 63.9 GiB RAM, 32 logical CPUs; Chromium 156.0.8078.4 and Firefox 157.0 | `pnpm --filter @xezim-ide/mcp-bridge profile:browser-memory -- chromium` and `-- firefox`; see `docs/luna-10-memory-report.md`. Does not qualify lower-resource devices or other OS/browser versions. |
| Official MCP Inspector CLI | Pass; targeted Inspector test ran | `pnpm --filter @xezim-ide/mcp-bridge test:inspector`; see `docs/luna-10-memory-report.md`. |
| Codex CLI ↔ browser pairing, edit, and run | Pass with Codex CLI `0.163.0` and Chromium `156.0.8078.4` | `pnpm --filter @xezim-ide/mcp-bridge smoke:codex`; ephemeral CLI config, temporary project, real Xezim WASM simulation. |
| Native Xezim core suite on current branch | Pass in Linux builder | `cargo test --manifest-path vendor/xezim-core/Cargo.toml -- --test-threads=1`; full results are recorded in `docs/luna-10-memory-report.md`. |
| Native Xezim suite on current branch | Pass in Linux builder; 5,226 passed, 1 perf test ignored | `cargo test --manifest-path vendor/xezim/Cargo.toml`. |
| Native xezim-web wrapper suite on current branch | Pass in Linux builder; 11 tests | `cargo test --manifest-path crates/xezim-web/Cargo.toml`. |
| Official CI workflow | Pending; GitHub access blocked in the current validation environment | `.github/workflows/luna09-browser-e2e.yml`; `gh auth status` reports an invalid token and `gh run list` could not resolve `api.github.com`. |
| License/distribution review | Compiler license texts and asset hashes are now served; the source branch is public in [PR #1](https://github.com/hevangel/xezim-studio/pull/1), while component/build provenance remains incomplete, hosted production URL checks, and qualified review are pending | See [Surfer distribution review](surfer-distribution-review.md) and [`apps/ide/public/licenses/THIRD_PARTY.md`](../apps/ide/public/licenses/THIRD_PARTY.md). browsercc's WASI sysroot tar has no notice files, and its build container uses mutable Emscripten `latest`. |

## Implemented bounds and remaining memory risk

- Xezim Worker inputs: 4,096 files and 16 MiB total UTF-8 paths plus file contents.
- MCP selected-project snapshot: 4,096 files, 64 MiB total, and 16 MiB per file.
- Waveform store: 64 MiB per VCD, at most 8 retained artifacts and 128 MiB aggregate. OPFS quota errors are surfaced.
- Xezim simulation stop time: 10,000,000 integer nanoseconds. Worker wall-time termination: 300,000 ms maximum.
- Browser console capture is limited to 1 MiB while Xezim records output; bounded mode coalesces messages into one transcript record and stops retaining output after the cap. The existing native entry points remain unbounded.
- Browser origin storage usage and quota are shown when the Storage API provides them. Persistent storage is requested with a bounded wait and is not assumed granted.
- WASM linear memory is capped at 256 MiB by the target linker maximum; allocation traps near the ceiling map to `RESOURCE_LIMIT`. This does not bound total browser process memory or impose a per-event/instruction count. The selected baseline was profiled, but broad large-design performance claims still need workload-specific evidence.

## Security and compatibility notes

- The MCP WebSocket listener binds to loopback and requires the pairing token and expected browser Origin. Project paths are normalized and symlink escapes rejected.
- Project writes and snapshots serialize through one mutation queue; snapshot reads verify revisions before and after collecting file contents.
- No browser-only startup path requires the bridge. Missing WASM reports unavailable and does not run native simulation.
- Native Xezim defaults remain enabled; the WASM feature/target gates are isolated. The CI native suite is the regression check for vendored target changes.
- Surfer v0.7.0 is bundled same-origin with its EUPL-1.2 notice. See [`v1-upgrade-notes.md`](v1-upgrade-notes.md) for provenance and distribution review notes.

## Next release gates

1. Review and merge [PR #1](https://github.com/hevangel/xezim-studio/pull/1) after its checks pass; run `.github/workflows/luna09-browser-e2e.yml` and retain hosted logs.
2. For the chosen public hosted website, publish the versioned source/provenance, configure the hosting project, record its canonical origin, and verify the license, source note, modified HTML entrypoint, and upstream source remain public while the viewer is served. No hosting manifest or canonical origin is configured in this checkout.
3. Obtain qualified review of the Surfer bundle, compiler toolchain assets, and combined application's license position. The [technical inventories](surfer-distribution-review.md) and [`apps/ide/public/licenses/THIRD_PARTY.md`](../apps/ide/public/licenses/THIRD_PARTY.md) are evidence for review, not legal approval.
