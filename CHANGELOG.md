# Changelog

Changes to HDL Workbench are recorded here.

## Unreleased — v1 hardening

- Added a New file action and a right-click Rename/Delete menu in the explorer, a waveform dump selector (VCD, FST, Off) and a simulator Arguments field for `+plusargs` and `-sv_seed`. FST is converted from the simulator's VCD inside the WASM module; the MCP `run_simulation` tool takes `plusargs` and `waveform_format`.
- Fixed runs whose design never calls `$dumpvars` failing with "Xezim did not return a supported VCD timescale".
- Added authenticated MCP-to-browser execution and persisted VCD viewing in the integrated browser workflow.
- Added revision-serialized project writes, snapshot consistency checks, project input bounds, waveform retention bounds, and origin storage estimates.
- Added Chromium and Firefox browser acceptance jobs and native Xezim compatibility suites to CI.
- Added reproducible Xezim WASM build and local demo commands. The browser simulator remains the pinned Xezim WASM Worker; the native simulator is not a browser fallback.
- Documented v1 upgrade expectations, unsupported browser capabilities, release evidence, and the Surfer EUPL-1.2 notice location.
