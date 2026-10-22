# ADR-001 — Browser simulation runtime boundary

Date: 2026-10-21
Status: Provisional; interface contracts remain unfrozen
Related work: LUNA-00A WASM feasibility and LUNA-00B Surfer integration

## Context

The product requires simulation to execute Xezim WebAssembly in a dedicated browser Worker. Native Xezim can serve as a differential-test oracle, but a native process cannot substitute for the browser simulator. The source inspection pins Xezim 22faa127a9cc0156bd23e0ab117f400c64231913 with xezim-core d15cdc39e27a4c0a8a55f7e1d46369181307ead9. The environment lacked Cargo and rustc, so the WASM compile gate did not reach the compiler. Surfer browser integration is being tested independently in LUNA-00B.

## Provisional decision

Retain the product boundary of Xezim WASM in a dedicated Worker, with no implicit native or JavaScript simulation fallback. Scope the first browser profile to bounded one-shot RTL simulation and VCD output. Keep native Xezim behavior as the default native profile. Do not claim browser support until the pinned source builds and a real browser counter fixture passes.

The implementation should call an explicit single-thread browser entry point, route project files through an injected virtual-file provider, route logs through an injected capture sink, and write VCD into a bounded in-memory sink. Existing Xezim APIs and native file behavior should remain available.

## Consequences

- Native DPI/VPI, libffi/libloading, JIT/AOT, arbitrary host file access, FST/XTrace, profiling and pause/resume are outside the initial browser profile unless later feasibility tests prove otherwise.
- Parser includes, $readmem*, $fopen, and logging need explicit supported behavior before the UI can advertise them.
- The SimulationService and SurferAdapter shapes in IMPLEMENTATION_PLAN.md remain proposals. Freeze them only after reviewing the LUNA-00B browser integration report and a runnable Xezim target result.
- LUNA-00A currently supports a patch plan, not a “WASM works” claim. Repeat compilation on a Rust-enabled host before starting the permanent Xezim refactor.
