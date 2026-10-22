# LUNA-03 — Xezim wasm worker

Date: 2026-10-22
Branch: `codex/luna-03-wasm-simulator`
Status: debug and optimized wasm builds plus real Chromium Worker acceptance pass. The browser backend is still a low-level one-shot runner and is not yet an application-facing `SimulationService`.

## Pins and build environment

| Component | Pin / observed version |
| --- | --- |
| Xezim | `22faa127a9cc0156bd23e0ab117f400c64231913` (`0.11.2`) |
| xezim-core and parser | `d15cdc39e27a4c0a8a55f7e1d46369181307ead9` (`0.11.2`) |
| Rust / Cargo | `1.99.1` |
| wasm-bindgen / CLI | `0.2.129` |
| Browser acceptance | Chromium `155.0.8059.79` on Debian 13 |

The source pair and upstream licenses are recorded in [`vendor/UPSTREAM.md`](../vendor/UPSTREAM.md). The wrapper lockfile is `crates/xezim-web/Cargo.lock`; the browser clock shim resolves `web-time 1.1.0`.

## Changes and portability findings

- Added `crates/xezim-web`, a wasm-bindgen `compileAndRun` wrapper that passes source text to pinned Xezim and returns JSON diagnostics, captured output, finish/fatal state, and simulation time. It invokes `simulate_multi_inline`; it does not invoke native simulation or shell out.
- Added a dedicated Vite module Worker and a one-request client that correlates ids, enforces a 30-second default / 5-minute maximum wall guard, terminates workers after a response, timeout, or crash, and maps failures to protocol error codes.
- Kept native defaults intact. `native-allocator` stays enabled by default in Xezim; the browser consumer disables that feature. `libffi` and `libloading` are native-target dependencies, JIT dependencies stay optional, and wasm build.rs exits before native C-shim, linker, and UVM checkout work.
- Added target-gated wasm paths for `getrandom`, 32-bit assertion-stat keys, and browser clocks. `web_time::Instant` and `js_sys::Date` are used only for wasm timing; native targets continue to alias `std::time` and `SystemTime`.
- Wasm selects Xezim’s existing inline stdout sink and inline simulation path. Native stdout and simulator worker threads remain enabled according to the existing native configuration.
- Initial real-browser attempts exposed two runtime traps that `cargo check` could not reveal: `SystemTime::now()` panicked on `wasm32-unknown-unknown`, and Xezim’s default stdout sink attempted to spawn a native thread. Both were patched at their target boundaries. A wasm32 shift of 40 bits in a `usize` assertion key was also caught by `cargo build` and given a single-source wasm representation.
- wasm compilation emits expected unused-code warnings for native-only DPI/VPI/FST/profiling paths. These warnings are not evidence those capabilities work in the browser.

## Reproduction

Run from the project root with Rust `1.99.1`, `wasm32-unknown-unknown`, clang (needed by the zstd build dependency), and `wasm-bindgen-cli 0.2.129`:

```sh
cargo build --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir apps/ide/public/wasm \
  crates/xezim-web/target/wasm32-unknown-unknown/debug/xezim_web.wasm
cargo build --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir apps/ide/public/wasm \
  crates/xezim-web/target/wasm32-unknown-unknown/release/xezim_web.wasm
pnpm --filter @xezim-ide/app typecheck
pnpm --filter @xezim-ide/app test
pnpm --filter @xezim-ide/app build
```

Generated wasm and glue files under `apps/ide/public/wasm/` are ignored by git. The browser acceptance page is `apps/ide/dist/worker-harness.html`. Serve the production build, launch Chromium with remote debugging, then run:

```sh
python3 -m http.server 4173 --directory apps/ide/dist
chromium --headless --no-sandbox --disable-dev-shm-usage \
  --remote-debugging-port=9222 --remote-allow-origins=* \
  --user-data-dir=/tmp/xezim-chrome about:blank
node apps/ide/scripts/wasm-browser-acceptance.mjs
```

Run the server and browser in separate terminals. On Node 20, add `--experimental-websocket` to the Node command; Node 24+ provides WebSocket globally. Set `XEZIM_CHROME_DEVTOOLS` or `XEZIM_BROWSER_TEST_URL` to override the defaults. The runner opens a fresh page, disables the browser cache, and checks the harness result.

The wasm wrapper tests and native Xezim suites must run on a Linux filesystem in this Windows environment. A Windows bind-mounted checkout caused `libffi-sys` Autoconf config probes to fail, so the checkout was copied to `/tmp/xezim-studio-native` inside the disposable Debian container before running native Cargo tests. From that scratch copy:

```sh
cargo test --offline --manifest-path crates/xezim-web/Cargo.toml
cargo test --offline --manifest-path vendor/xezim-core/Cargo.toml
cargo test --offline --manifest-path vendor/xezim/Cargo.toml
```

## Acceptance results

- `cargo check --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown --lib` — passed during the portability iteration.
- `cargo build --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown` — passed after the target patches.
- `cargo build --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown --release` — passed; the raw release artifact is 18,569,703 bytes, and generated `xezim_web_bg.wasm` is 17,999,491 bytes (about 17.2 MiB).
- wasm-bindgen generation with CLI `0.2.129` — passed.
- Browser acceptance in Chromium — passed against both debug and optimized artifacts. It ran the real four-bit counter in a dedicated Worker, returned `COUNT=4` at `42000` time ticks, returned a structured `failed` result with diagnostics for malformed SystemVerilog, then ran the counter successfully in a fresh Worker.
- `pnpm test` — passed, 27 tests (19 protocol tests and 8 app tests).
- `pnpm typecheck` and `pnpm build` — passed. Vite reports the existing large Monaco app chunk (>500 kB).
- `pnpm --filter @xezim-ide/app typecheck` — passed.
- `pnpm --filter @xezim-ide/app test` — passed, 8 tests across 2 files (includes timeout/crash correlation and fresh-worker recovery tests).
- Native wrapper tests — passed, 3 tests.
- `cargo test --offline --manifest-path vendor/xezim-core/Cargo.toml` — passed (the crate currently has no standalone tests).
- `cargo test --offline --manifest-path vendor/xezim/Cargo.toml` — passed, exit code 0 after the final wasm-targeted changes. All unit, integration, UVM, and doc-test binaries completed without failures; cargo printed per-binary counts.

## Current browser limits and blockers

At the LUNA-03 milestone, the wrapper deliberately supported one source string and one top module. It enforced a 4 MiB UTF-8 source limit, 512-byte top-module limit, 10,000,000 ns simulation-time limit, and 1 MiB returned-output limit. The output cap was applied after Xezim had accumulated output, so it did not cap peak output memory during simulation. The Worker wall guard could terminate a stuck simulation, but this milestone did not enforce the protocol's `maxSteps` budget or a fixed wasm linear-memory ceiling. LUNA-10 later added a 256 MiB WebAssembly memory maximum and classified allocation traps near the ceiling; output accumulation and per-event budgeting remain open. See the [LUNA-10 memory report](luna-10-memory-report.md).

The debug wasm artifact was 69,594,804 bytes (about 66.4 MiB); the optimized generated artifact is 17,999,491 bytes (about 17.2 MiB). No explicit wasm stack-size flag is set and no deep-recursion stress test has been done. LUNA-10 later added a 256 MiB linear-memory maximum and workload-specific memory measurements; browser stack behavior and repeat memory stress on a minimum-supported device remain unverified.

This worker is not the frozen protocol's `SimulationService`: it combines compile and run, takes source text directly, uses nanoseconds rather than protocol ticks plus an explicit timescale, and does not carry project/snapshot/run ids or emit a VCD artifact. Keep the UI, MCP bridge, and capability report on their existing mock/unavailable paths until a protocol adapter and the LUNA-04 virtual-file/VCD work pass their gates.

Also unsupported in this browser profile: `include`, `$readmem*`, `$fopen` and other host file tasks, VCD/FST writing, DPI/VPI dynamic loading, native JIT/AOT, and native thread-backed features. No browser capability should be advertised for them.

## Recommended next tasks

1. Measure peak linear memory and stress browser stack behavior on Chromium before shipping the optimized artifact.
2. Coordinate LUNA-04’s virtual file and inline VCD work, then add a typed adapter that maps the pinned wrapper to the frozen compile/run protocol. Until then this branch proves wasm feasibility, not application-service conformance.
