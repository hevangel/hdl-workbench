# ADR-001 — Browser simulation runtime boundary

Date: 2026-10-21

Status: Accepted for the product boundary; runtime conformance gates remain open

Related work: [ADR-002 interface freeze](ADR-002-interface-contracts.md), [LUNA-00A report](wasm-portability-report.md), [LUNA-00B report](surfer-integration-report.md)

## Context

The product requires simulation to execute Xezim WebAssembly in a dedicated browser Worker. Native Xezim can serve as a differential-test oracle, but a native process cannot substitute for the browser simulator. LUNA-00A pins Xezim `22faa127a9cc0156bd23e0ab117f400c64231913` with xezim-core `d15cdc39e27a4c0a8a55f7e1d46369181307ead9`. The source inventory found native build and runtime boundaries, but Cargo and rustc were unavailable, so the WASM compile gate did not reach the compiler.

LUNA-00B pins Surfer source `bd749b1f786c1c62cd67893ca71346cbe6983915` (v0.7.0, EUPL-1.2). Its source documents `LoadUrl` and an unstable `InjectMessage` forwarding path. The versioned WASM artifact was unavailable, and no browser run verified waveform loading, signal addition, or cursor control.

## Decision

Keep the product boundary as Xezim WASM in a dedicated browser Worker, with no implicit native or JavaScript simulation fallback. Scope the first browser profile to bounded one-shot RTL simulation and VCD output. Keep native Xezim behavior as the default native profile.

Freeze application-facing interface shapes only as described in ADR-002. Keep concrete wasm-bindgen signatures, Xezim internal file-provider APIs, Surfer message payloads, readiness wire protocol, and waveform URL transport behind their adapters until the feasibility gates pass.

Do not claim browser support until the pinned Xezim source builds and a real browser Worker counter fixture passes. Do not claim Surfer signal/cursor control until the pinned and licensed viewer build passes a browser test for loading a VCD, adding a signal, and moving a cursor.

## Consequences

- Native DPI/VPI, libffi/libloading, JIT/AOT, arbitrary host file access, FST/XTrace, profiling and pause/resume are outside the initial browser profile unless later feasibility tests prove otherwise.
- Parser includes, `$readmem*`, `$fopen`, and logging need explicit supported behavior before the UI can advertise them.
- Optional simulation operations such as cancellation, current-value lookup, and historical transition queries are not part of the v1 service contract. Add them only with implementations and acceptance tests.
- The initial browser runtime must call an explicit single-thread entry point, route project files through a virtual-file provider, capture logs through a sink, and write VCD into a bounded in-memory sink. Existing Xezim APIs and native file behavior must remain available.
- The application-facing contracts are frozen, but the backend feasibility and browser integration gates remain open. No feature is considered implemented by the interface definition alone.
