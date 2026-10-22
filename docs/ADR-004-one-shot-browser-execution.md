# ADR-004 — Additive one-shot browser execution RPC

Date: 2026-10-22

Status: Accepted for LUNA-09 integration

Related decisions: [ADR-001 runtime architecture](ADR-001-runtime-architecture.md), [ADR-002 interface freeze](ADR-002-interface-contracts.md)

## Context

The pinned browser binding exposes `compileAndRunProject(files, snapshotId, topModule, maxTimeNs, maxOutputBytes, maxVcdBytes)` as one bounded Worker call. It does not expose a compiled design handle that survives a Worker call. Mapping `simulation.compile` to a time-zero run would execute SystemVerilog side effects and misstate the result. Mapping `simulation.run` to a second one-shot call would compile twice and would not represent the same compiled design.

The application still needs one real operation shared by the Run button and authenticated MCP requests. The bridge and browser can both bind that operation to an immutable snapshot. The browser Worker owns the compile-and-simulate operation; the Node bridge remains a transport and local project-file authority, never a native simulation backend.

## Decision

Add `simulation.execute` to protocol `xezim-ide/1` with `projectId`, `snapshotId`, `topModule`, `until`, and only the wall-time, stdout, and VCD limits the browser API can enforce. Its result is the existing `RunResult` with matching snapshot and waveform provenance.

Keep `simulation.compile` and `simulation.run` in the schema for other backends and future Xezim APIs. The current browser controller returns `UNSUPPORTED_CAPABILITY` for both. The MCP `run_simulation` tool accepts `top_module` and stop-time/budget inputs and calls `simulation.execute`; it does not require or fabricate a compile ID. MCP callers can still see the separate `compile` tool, but it reports the current browser limitation clearly.

The Run button calls the same browser controller `execute()` method as the authenticated RPC dispatcher. `simulation.execute` first recomputes the workspace snapshot from project file revisions, then passes only that snapshot and request limits to the Xezim Worker. Complete VCD output is stored in OPFS and Surfer receives a same-origin object URL resolved from artifact metadata. No VCD bytes are placed in MCP JSON.

## Known limits

- The Xezim Worker currently accepts integer nanosecond stop times up to 10,000,000 ns. Requests that would need fractional-nanosecond rounding are rejected.
- The current browser API cannot enforce `maxSteps`, cancellation inside Xezim, or arbitrary VCD scope selection. The runtime has a fixed 256 MiB WebAssembly linear-memory maximum from target linker settings, not a per-request memory field; the Worker reports allocation traps near the ceiling as `RESOURCE_LIMIT`. Wall-time termination is enforced by terminating the Worker. No per-event or instruction-count budget is available.
- Existing separate compile/run RPC operations remain unavailable in this browser implementation.
- The Surfer adapter confirms dispatch after readiness but has no asynchronous VCD parse-completion acknowledgment.

## Validation

Contract tests check strict request shape and snapshot correlation. Browser-controller tests check stale snapshot rejection, no native fallback, explicit unsupported compile/run, time conversion, Worker result mapping, and VCD artifact persistence. The LUNA-09 report records the bridge and browser test results and any remaining end-to-end gate.
