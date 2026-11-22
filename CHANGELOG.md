# Changelog

Changes to HDL Workbench are recorded here.

## Unreleased

## 1.1.0 — 2026-11-22

- Ignored the local `.kilo/` configuration directory.
- Fixed the native build by removing the unused `rand` dependency from the vendored xezim-core.
- Raised the hdlwb CLI session test timeout so it is not flaky on loaded CI runners.

## 1.0.0 — 2026-11-22

- Updated Xezim to 0.11.7 and xezim-core to 0.11.7 (Rust 1.100). Surfer stays at v0.7.0, the latest upstream release.
- Added a New file action and a right-click Rename/Delete menu in the explorer, a waveform dump selector (VCD, FST, Off) and a simulator Arguments field for `+plusargs` and `-sv_seed`. FST is converted from the simulator's VCD inside the WASM module; the MCP `run_simulation` tool takes `plusargs` and `waveform_format`.
- Fixed runs whose design never calls `$dumpvars` failing with "Xezim did not return a supported VCD timescale".
- Added authenticated MCP-to-browser execution and persisted VCD viewing in the integrated browser workflow.
- Added revision-serialized project writes, snapshot consistency checks, project input bounds, waveform retention bounds, and origin storage estimates.
- Added Chromium and Firefox browser acceptance jobs and native Xezim compatibility suites to CI.
- Added reproducible Xezim WASM build and local demo commands. The browser simulator remains the pinned Xezim WASM Worker; the native simulator is not a browser fallback.
- Documented v1 upgrade expectations, unsupported browser capabilities, release evidence, and the Surfer EUPL-1.2 notice location.
