# Changelog

Changes to Xezim Studio are recorded here.

## Unreleased — v1 hardening

- Added authenticated MCP-to-browser execution and persisted VCD viewing in the integrated browser workflow.
- Added revision-serialized project writes, snapshot consistency checks, project input bounds, waveform retention bounds, and origin storage estimates.
- Added Chromium and Firefox browser acceptance jobs and native Xezim compatibility suites to CI.
- Added reproducible Xezim WASM build and local demo commands. The browser simulator remains the pinned Xezim WASM Worker; the native simulator is not a browser fallback.
- Documented v1 upgrade expectations, unsupported browser capabilities, release evidence, and the Surfer EUPL-1.2 notice location.
