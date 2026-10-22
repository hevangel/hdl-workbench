# v1 release readiness

Date: 2026-10-22
Branch: `codex/luna-10-hardening`
Status: branch-local hardening and acceptance checks pass; external release gates remain.

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
| End-to-end MCP edit → Xezim WASM → OPFS VCD → Surfer → reload | Pass in Chromium `156.0.8078.4` | `pnpm test:e2e`; ROM plus counter run. |
| End-to-end in Firefox | Pass in Firefox `157.0` | `pnpm test:e2e` runs both configured browser projects. |
| `pnpm typecheck` | Pass | All four workspace packages. |
| `pnpm build` | Pass; existing Monaco bundle size warning | All workspace packages and generated protocol JSON schema. |
| `pnpm test` | Pass with local loopback permission; 62 tests | Protocol 20, IDE 29, MCP bridge 7, wave-index 6. |
| Capped `pnpm build:browser` equivalent | Pass in the pinned Linux builder | `cargo build --locked --release --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown`, then `wasm-bindgen --target web --out-dir apps/ide/public/wasm crates/xezim-web/target/wasm32-unknown-unknown/release/xezim_web.wasm`; see `docs/luna-10-memory-report.md`. |
| Concurrent same-revision bridge writes | Covered by regression test | `pnpm test`; exactly one concurrent write succeeds. |
| Source project and waveform bounds | Implemented with regression coverage | `pnpm test`; see limits below. |
| Full Chromium + Firefox E2E on current branch | Pass; 4 cases across 2 browsers, 22.6 seconds | `pnpm test:e2e`; MCP edit/run, ROM/NBA, Surfer waveform/reload, memory ceiling, and worker recovery. |
| WASM memory ceiling and failure recovery | Pass in Chromium `156.0.8078.4` and Firefox `157.0` | `pnpm --filter @xezim-ide/mcp-bridge exec playwright test e2e/luna10-memory.spec.mjs --project=chromium` and `--project=firefox`; see `docs/luna-10-memory-report.md`. |
| Official MCP Inspector CLI | Pass; targeted Inspector test ran | `pnpm --filter @xezim-ide/mcp-bridge test:inspector`; see `docs/luna-10-memory-report.md`. |
| Codex CLI ↔ browser pairing, edit, and run | Pass with Codex CLI `0.163.0` and Chromium `156.0.8078.4` | `pnpm --filter @xezim-ide/mcp-bridge smoke:codex`; ephemeral CLI config, temporary project, real Xezim WASM simulation. |
| Native Xezim core suite on current branch | Pass in Linux builder | `cargo test --manifest-path vendor/xezim-core/Cargo.toml -- --test-threads=1`; full results are recorded in `docs/luna-10-memory-report.md`. |
| Native Xezim suite on current branch | Pass in Linux builder | Complete `cargo test --manifest-path vendor/xezim/Cargo.toml -- --test-threads=1`; includes the 705-test integration target, 4 UPF tests, and doctests. |
| Native xezim-web wrapper suite on current branch | Pass in Linux builder; 7 tests | `cargo test --manifest-path crates/xezim-web/Cargo.toml --lib -- --test-threads=1`. |
| Official CI workflow | Pending GitHub execution | `.github/workflows/luna09-browser-e2e.yml`. |
| License/distribution review | Pending before external release | Surfer notice is retained; distribution model still requires review. |

## Implemented bounds and remaining memory risk

- Xezim Worker inputs: 4,096 files and 16 MiB total UTF-8 paths plus file contents.
- MCP selected-project snapshot: 4,096 files, 64 MiB total, and 16 MiB per file.
- Waveform store: 64 MiB per VCD, at most 8 retained artifacts and 128 MiB aggregate. OPFS quota errors are surfaced.
- Xezim simulation stop time: 10,000,000 integer nanoseconds. Worker wall-time termination: 300,000 ms maximum.
- Browser origin storage usage and quota are shown when the Storage API provides them. Persistent storage is requested with a bounded wait and is not assumed granted.
- WASM linear memory is capped at 256 MiB by the target linker maximum; allocation traps near the ceiling map to `RESOURCE_LIMIT`. This does not bound total browser process memory or impose a per-event/instruction count. Stress measurements on a minimum-supported browser/device remain a release gate for large designs.

## Security and compatibility notes

- The MCP WebSocket listener binds to loopback and requires the pairing token and expected browser Origin. Project paths are normalized and symlink escapes rejected.
- Project writes and snapshots serialize through one mutation queue; snapshot reads verify revisions before and after collecting file contents.
- No browser-only startup path requires the bridge. Missing WASM reports unavailable and does not run native simulation.
- Native Xezim defaults remain enabled; the WASM feature/target gates are isolated. The CI native suite is the regression check for vendored target changes.
- Surfer v0.7.0 is bundled same-origin with its EUPL-1.2 notice. See [`v1-upgrade-notes.md`](v1-upgrade-notes.md) for provenance and distribution review notes.

## Next release gates

1. Run the workflow on GitHub and retain its logs as release evidence; local Linux-builder results are documented, but the hosted CI run is still pending.
2. Define the minimum supported browser/device and measure total process memory and stress limits there; keep large designs out of v1 claims until measured.
3. Review the Surfer EUPL distribution obligations for the intended release format.
