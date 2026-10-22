# LUNA-00A — Xezim WASM portability report

Date: 2026-10-21
Branch: codex/luna-00a-wasm-feasibility
Worktree: C:/Users/hevan/AppData/Local/Temp/xezim-studio-luna/luna-00a

## Result

The upstream revisions are pinned and the source inventory identifies a plausible small RTL-only browser profile. A WASM build was **not verified**: this environment has no Rust toolchain, so each requested cargo check stopped before Cargo could read a manifest or compile source. This is an environment blocker, not evidence that the target builds or fails.

No Xezim source was changed. The code observations below are from the pinned upstream source, not a successful build.

## Pins

| Repository | Revision | Version | Evidence |
|---|---|---|---|
| Xezim | 22faa127a9cc0156bd23e0ab117f400c64231913 | 0.11.2 | git ls-remote and detached checkout; commit message “Sync core revision” |
| xezim-core | d15cdc39e27a4c0a8a55f7e1d46369181307ead9 | 0.11.2 | git ls-remote and detached checkout; commit message “Replicate four-state sign bits in arithmetic shifts (§11.4.10)” |

Xezim's Cargo.toml at 22faa pins both xezim-core and sv-parser to d15cdc39e27a4c0a8a55f7e1d46369181307ead9. Pin the pair together.

## Reproduction

The pinned source checkouts used for inspection were cloned outside the project tree. On a Rust 1.92+ host with the WASM target installed, these commands reproduce the source revisions and initial target checks:

    $upstream = Join-Path $env:TEMP 'xezim-upstream-luna00a'
    git clone https://github.com/aionhw/xezim.git "$upstream/xezim"
    git -C "$upstream/xezim" checkout --detach 22faa127a9cc0156bd23e0ab117f400c64231913
    git clone https://github.com/aionhw/xezim-core.git "$upstream/xezim-core"
    git -C "$upstream/xezim-core" checkout --detach d15cdc39e27a4c0a8a55f7e1d46369181307ead9
    rustup target add wasm32-unknown-unknown
    cargo check --manifest-path "$upstream/xezim-core/xezim-parser/Cargo.toml" --target wasm32-unknown-unknown
    cargo check --manifest-path "$upstream/xezim-core/Cargo.toml" --target wasm32-unknown-unknown --no-default-features
    cargo check --manifest-path "$upstream/xezim/Cargo.toml" --target wasm32-unknown-unknown --no-default-features --lib

Observed tool results in this environment:

- git ls-remote https://github.com/aionhw/xezim.git HEAD refs/heads/main returned 22faa127a9cc0156bd23e0ab117f400c64231913 for both refs.
- git ls-remote https://github.com/aionhw/xezim-core.git HEAD refs/heads/main returned d15cdc39e27a4c0a8a55f7e1d46369181307ead9 for both refs.
- cargo --version, rustc --version, and rustup target list --installed could not run: the commands are not installed or on PATH.
- All three cargo check attempts returned PowerShell's “The term 'cargo' is not recognized” with exit code 1. The attempts did not reach Cargo, so there are no Rust compiler diagnostics to classify.
- An isolated rustup provisioning attempt did not leave a usable toolchain in the temporary home. Run the checks on a Rust-enabled host before treating this gate as passed.

## Source inventory

### Parser

xezim-core/xezim-parser is a Rust library with only optional serde/serde_json dependencies. Its source parsing APIs accept source strings, which is a good fit for a browser Worker. The path-based parse_file helper reads from std::fs; include resolution in the preprocessor also reads files. The parser needs a virtual include resolver for multi-file browser projects, or the first smoke test must deliberately use one source file without includes.

### xezim-core

- The default mimalloc-allocator feature is optional. default-features = false avoids installing its C-backed global allocator in a WASM consumer.
- zstd is a required dependency and core code uses it for XTrace/compressed output. The browser RTL/VCD profile does not need that capability; make it optional behind a native/XTrace feature before concluding the core target is clean.
- Core contains host-file operations for serialization and source/path helpers. Those paths must be excluded from the browser surface or redirected through a provider.
- src/vcd_sink.rs already has VcdSink::inline(DumpWriter); it also has threaded and file-opening constructors. A bounded in-memory Write implementation can reuse the inline path without changing the native file sink. The writer type currently requires Write + Send.

### Xezim

- The crate is Rust edition 2024, MSRV 1.92, version 0.11.2. Its manifest has no browser feature.
- libffi, libloading, and libc are unconditional dependencies. DPI/VPI dynamic loading is native functionality and should not enter the browser profile. Cranelift dependencies are optional behind jit, but JIT/AOT remain unsupported in the browser MVP.
- src/build.rs always emits a native export-dynamic linker argument for non-Windows/non-macOS targets and compiles vpi_printf_shim.c and svdpi_shim.c by invoking cc and ar. This must be target-gated before wasm32 can be a valid crate build.
- Public simulate_multi defaults to a 1024 MiB Rust thread and uses XEZIM_STACK_MB=0 to run on the caller thread. That environment override is not a browser API. The scheduler also contains parallel settling/edge paths, profiling threads and threaded waveform writers. Browser mode needs a deterministic single-thread entry point and must keep internal parallel paths disabled.
- VCD setup in src/compiler/simulator/waveforms.rs currently creates a filesystem file and calls VcdSink::open_file; adapt this call site to an injected bounded inline writer for browser mode. Preserve the existing path-based native behavior.
- $readmem*, $fopen and file writes use std::fs in simulator code. Preprocessor includes and source paths also assume host files. These are runtime integration blockers even if the crates compile.
- Simulator output and diagnostics include direct stdout/stderr writes. The browser wrapper needs an injected output collector so logs and errors return as structured data rather than depending on host streams.
- Linux memory sampling reads /proc. Treat such telemetry as optional and compile it out or provide a no-op implementation on wasm.

## Blocker inventory

| Priority | Finding | Classification |
|---|---|---|
| P0 | No Rust toolchain in this environment; cargo checks stop before compilation. | Feasibility gate remains unverified. |
| P0 | Unconditional C shim compilation and native linker argument in xezim build.rs. | Source-confirmed build-script blocker for wasm32. |
| P0 | Mandatory libffi/libloading dependency path for DPI/VPI. | Native-only feature/dependency boundary; target compilation still needs a real check. |
| P1 | Default Xezim API uses OS thread creation; inner runtime has parallel and threaded paths. | Browser execution cannot rely on existing default path. |
| P1 | VCD output opens a host file; source includes and SV file tasks use host filesystem. | Browser runtime blocker; requires provider/output adaptation. |
| P1 | Core zstd dependency is required for non-MVP XTrace paths. | Likely unnecessary in the browser profile; compile impact unverified. |
| P2 | Native stdout/stderr and /proc sampling assumptions. | Must be gated/captured for a clean Worker API. |

“P0” here means a gate to resolve before claiming a browser simulator; it does not mean a compiler error was observed.

## Minimal patch strategy

1. Keep the upstream revisions above as an explicit pair and make browser support a small consumer feature/profile. Keep native features enabled by default for the existing CLI/library behavior; make browser-only dependencies and build steps target-gated.
2. In build.rs, skip native C shims and export-dynamic arguments on wasm32. Gate DPI/VPI, libffi/libloading, JIT/AOT, profiling and native-only output behind native features or target cfgs.
3. Add an explicit single-thread compile/run entry point that calls the existing inner simulator. Do not depend on XEZIM_STACK_MB; audit and gate all runtime thread pools, samplers and background sinks for the browser profile.
4. Add a project-relative virtual file provider at the parser/include and simulator file-I/O boundaries. Start with a deliberately declared RTL-only subset; implement include, $readmem*, and supported $fopen operations through that provider before claiming them.
5. Reuse VcdSink::inline with an in-memory writer that enforces a byte limit. Adapt waveform setup to accept that writer, while retaining native file output. Capture simulator messages through an injected sink.
6. Re-run the parser → core with defaults off → reduced Xezim sequence on a Rust 1.92+ host. Only after compile results, add a real browser Worker counter test and compare native versus WASM output. Keep native regression tests in the same patch set.

## Gate status and next tasks

- **Confirmed:** exact Xezim/core pins; parser is a source-string Rust library; core exposes an inline VCD sink; source-level native boundaries are located.
- **Not confirmed:** parser/core/xezim wasm compilation, browser Worker execution, VCD bytes, or native regression status.
- **Next:** rerun the three commands in the reproduction section on a Rust-enabled host and save full compiler output. If parser passes, tackle the build script and native feature boundaries before deep filesystem changes.
- **Interface freeze:** do not freeze SimulationService, waveform transport, or Surfer message adapter from this report alone. LUNA-00B is still outstanding, and this compile gate did not run.


## Pinned source links

- [Xezim commit 22faa127](https://github.com/aionhw/xezim/commit/22faa127a9cc0156bd23e0ab117f400c64231913)
- [Xezim Cargo.toml](https://github.com/aionhw/xezim/blob/22faa127a9cc0156bd23e0ab117f400c64231913/Cargo.toml)
- [Xezim build script](https://github.com/aionhw/xezim/blob/22faa127a9cc0156bd23e0ab117f400c64231913/src/build.rs)
- [Xezim simulation entry points](https://github.com/aionhw/xezim/blob/22faa127a9cc0156bd23e0ab117f400c64231913/src/lib.rs)
- [Xezim waveform setup](https://github.com/aionhw/xezim/blob/22faa127a9cc0156bd23e0ab117f400c64231913/src/compiler/simulator/waveforms.rs)
- [xezim-core commit d15cdc39](https://github.com/aionhw/xezim-core/commit/d15cdc39e27a4c0a8a55f7e1d46369181307ead9)
- [xezim-core Cargo.toml](https://github.com/aionhw/xezim-core/blob/d15cdc39e27a4c0a8a55f7e1d46369181307ead9/Cargo.toml)
- [xezim-core inline VCD sink](https://github.com/aionhw/xezim-core/blob/d15cdc39e27a4c0a8a55f7e1d46369181307ead9/src/vcd_sink.rs)
- [sv-parser Cargo.toml](https://github.com/aionhw/xezim-core/blob/d15cdc39e27a4c0a8a55f7e1d46369181307ead9/xezim-parser/Cargo.toml)
