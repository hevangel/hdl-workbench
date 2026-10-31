# Protocol changelog

## xezim-ide/1 — 2026-10-22

- Added `simulation.execute`, an additive one-shot compile-and-run operation for the pinned browser Xezim API. See [ADR-004](../../session_history/initial_build/ADR-004-one-shot-browser-execution.md).
- The operation binds the result to the requested project snapshot and exposes only the wall-time, output, and VCD limits currently enforced by the browser Worker.
- Existing `simulation.compile` and `simulation.run` remain in the schema; the current browser controller reports them as unsupported instead of simulating them approximately.
