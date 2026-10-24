# LUNA-04 — Project virtual files and bounded VCD output

Date: 2026-10-22
Branch: `codex/luna-04-virtual-files`
Status: implementation and browser acceptance passed on the pinned Xezim source pair.

## Scope and source pins

| Component | Revision / version |
| --- | --- |
| Xezim | `22faa127a9cc0156bd23e0ab117f400c64231913` (`0.11.2`) |
| xezim-core and sv-parser | `d15cdc39e27a4c0a8a55f7e1d46369181307ead9` (`0.11.2`) |
| Rust / Cargo | `1.99.1` |
| wasm-bindgen / CLI | `0.2.129` |

The vendored sources are the pinned archives documented in [`vendor/UPSTREAM.md`](../../../vendor/UPSTREAM.md); LUNA-03 portability changes and this VFS patch are applied in place. The native APIs remain filesystem-backed by default.

## Implemented surface

- Added `compileAndRunProject(filesJson, snapshotId, topModule, maxTimeNs, maxOutputBytes, maxVcdBytes)` to `xezim-web`. The Worker accepts named UTF-8 project files, passes `.sv` files as ordered compilation units, and makes the complete file map available to includes and runtime file tasks.
- Added `xezim::simulate_project_inline` and `xezim_core::preprocess_design_with_virtual_files`. Virtual includes are resolved from project-relative paths and cannot fall through to host files. Absolute paths, drive paths, `..`, and duplicate normalized paths are rejected.
- Routed `$readmem*` and the supported `$fopen` read/write modes through the in-memory VFS. Text output such as `$fdisplay` / `$fgets` is returned as changed project files. `$system` produces `XEZIM_WEB_UNSUPPORTED_TASK` in browser runs and project-VFS runs; it is not invoked.
- Reused `VcdSink::inline` with an injected bounded writer. Project runs return VCD bytes, format, byte count, completeness, timescale, and snapshot ID. VCD overflow is reported as a resource-limit result without returning a partial waveform. Native runs keep the original file-backed VCD writer.
- Captured `$display` in the browser result without writing to process stdout. The prior `compileAndRun` entry point now runs through a one-file virtual project, so the browser path does not regain host filesystem access.
- Replaced the wasm `$date` clock call with `web_time::SystemTime`; native VCD timestamps still use `std::time::SystemTime`. The first real browser project run found this wasm-only trap because Xezim's standard-library `SystemTime::now()` panics on `wasm32-unknown-unknown`.

## Limits and current boundaries

- Project inputs: 16 MiB total; VCD: 64 MiB; returned stdout: 1 MiB; simulation time: 10,000,000 ns. The Worker also rejects more than 4,096 project files.
- Changed VFS file output is UTF-8-lossy text in this JSON API. Binary file output is not a supported browser contract.
- `$fopen` supports the modes and text operations exercised by the fixture; this is not full IEEE file-task parity. Unsupported tasks must remain explicit diagnostics.
- The wrapper combines compile and run and takes nanoseconds. It is not yet a full `SimulationService` implementation with separate compile/run handles, max-step accounting, cancellation, or protocol tick/time-unit conversion.
- At the LUNA-04 milestone, the stdout cap was applied to the returned string after Xezim accumulated output, and no fixed WASM linear-memory ceiling was enforced. LUNA-10 later added a 256 MiB WebAssembly memory maximum and trap classification; output accumulation itself remains unbounded by that stdout cap. See the [LUNA-10 memory report](luna-10-memory-report.md).
- The generated wasm and JS glue under `apps/ide/public/wasm/` are build artifacts ignored by Git. Regenerate them from the pinned source before serving a clean checkout.

## Reproduction

Use Rust `1.99.1`, the `wasm32-unknown-unknown` target, clang (required by `zstd-sys`), and matching `wasm-bindgen-cli 0.2.129`:

```sh
rustup target add wasm32-unknown-unknown
cargo test --manifest-path crates/xezim-web/Cargo.toml --lib -- --test-threads=1
cargo test --manifest-path vendor/xezim/Cargo.toml --lib -- --test-threads=1
cargo check --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown
cargo build --release --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir apps/ide/public/wasm \
  crates/xezim-web/target/wasm32-unknown-unknown/release/xezim_web.wasm
pnpm --filter @xezim-ide/app test
pnpm --filter @xezim-ide/app build
```

In this Windows workspace, native Cargo tests were run from a Linux scratch copy because the Windows bind mount prevents `libffi-sys` Autoconf probes from creating `config.log`. With the Debian 13 builder container `xezim-luna04-builder` mounted at `/host`, refresh the scratch source and run the complete native acceptance suite with:

```sh
docker exec xezim-luna04-builder sh -lc 'tar --exclude=target --exclude=.git -cf - -C /host crates/xezim-web vendor/xezim vendor/xezim-core .cargo rust-toolchain.toml | tar -xf - -C /tmp/xezim-luna04 && cd /tmp/xezim-luna04 && /usr/local/cargo/bin/cargo test --manifest-path vendor/xezim/Cargo.toml -- --test-threads=1'
```

`zstd-sys` also required clang for its wasm C build. The wasm source check and release build passed after clang was installed in the Debian 13 builder.

The standard Chromium runner is `apps/ide/scripts/wasm-browser-acceptance.mjs`; serve `apps/ide/dist` on port 4173 and provide a Chromium DevTools endpoint at port 9222. The available browser here was the existing Camoufox Playwright server instead of Chromium. The exact browser check used `playwright.firefox.connect('ws://127.0.0.1:9222/hkej')`, loaded `http://127.0.0.1:4173/worker-harness.html`, and inspected `#result[data-status]`. Since the browser is isolated in a container, the built `apps/ide/dist` contents were streamed into that container and served on its loopback port 4173.

## Acceptance results

| Check | Result |
| --- | --- |
| `cargo test --manifest-path crates/xezim-web/Cargo.toml --lib -- --test-threads=1` | Passed: 7 tests, including multi-file compilation, nested includes, `$readmemh`, `$fopen`, four-state values, NBA ordering, VCD transitions, output/VCD caps, path traversal, missing ROM, and `$system`. |
| `cargo test --manifest-path vendor/xezim/Cargo.toml --lib -- --test-threads=1` | Passed: 88 tests. |
| `cargo test --manifest-path vendor/xezim-core/Cargo.toml --lib -- --test-threads=1` | Passed: 71 tests. |
| `cargo test --manifest-path vendor/xezim/Cargo.toml -- --test-threads=1` | Passed: the complete native Xezim suite, including every integration target and doctests. The first run exposed two native include failures; separating native and virtual-only preprocessor setup fixed them, and both affected include tests passed before the complete rerun. |
| `cargo check --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown` | Passed after adding clang to the Linux builder. It reports unused native DPI/VPI/FST code warnings; those paths are not browser capabilities. |
| Release wasm build and `wasm-bindgen --target web` | Passed. Generated wasm SHA-256: `2e9b13251ea82b4569e4c6469ec72009eef0d0af863d63020bffef8ff056a960` (18 MiB). |
| `pnpm --filter @xezim-ide/app test` | Passed: 9 tests across 2 files. |
| `pnpm --filter @xezim-ide/app build` | Passed, including TypeScript typecheck. Vite reports the existing large Monaco application chunk. |
| Real browser worker harness | Passed against the final wasm in Firefox `152.0` (Windows user agent). Counter returned `COUNT=4`; the multi-file project returned `completed`, snapshot `sha256:luna-04-browser-fixture`, complete 699-byte VCD at `1ps`, expected transitions at ticks 0/1000/3000, X/Z `10xz`, expected NBA ordering and ROM output, and `captured.txt`. Malformed SV returned a diagnostic and a fresh counter worker then recovered successfully. |
| `rustfmt --check` on changed Rust files; `git diff --check` | Passed. |

The native VFS wrapper test and browser harness check the same expected counter transitions (`1`, `2`, `3` at the corresponding edge samples), and the browser additionally verifies the serialized VCD. The full native Xezim suite confirms the default filesystem-backed include and simulation paths still pass.

## Recommended next tasks

1. LUNA-05 now embeds the pinned Surfer bundle and passes load, signal-add, cursor, zoom, and iframe reload in Camoufox Firefox. See the [LUNA-05 report](luna-05-surfer-adapter-report.md). Verify production Chromium, large-waveform performance, WCP exports, and artifact-load completion semantics before release.
2. LUNA-06 persistence passes the available Firefox workflow; verify ordinary page-entry startup in supported browsers and exercise physical quota/eviction behavior.
3. Implement LUNA-07 MCP bridge and LUNA-08 deterministic historical VCD queries, then wire both through LUNA-09 E2E without enabling unsupported operations.
4. Add memory/work-budget enforcement and complete the Xezim `SimulationService` mapping. Keep the passing full native Xezim suite as a compatibility gate for future browser-only changes.
