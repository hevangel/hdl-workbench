# ADR-001 — Browser simulation runtime boundary

Date: 2026-10-22

Status: Accepted after LUNA-00A and LUNA-00B feasibility review; product integration remains partial

Related work: [ADR-002 interface freeze](ADR-002-interface-contracts.md), [Xezim portability report](wasm-portability-report.md), [Xezim build-gate follow-up](wasm-build-gate-followup.md), [Surfer feasibility report](surfer-integration-report.md), [LUNA-04 implementation report](luna-04-virtual-files-report.md)

## Context

The product requires simulation to execute Xezim WebAssembly in a dedicated browser Worker. Native Xezim can serve as a differential-test oracle, but a native process cannot substitute for the browser simulator. The reviewed upstream pair is Xezim `22faa127a9cc0156bd23e0ab117f400c64231913` and xezim-core/sv-parser `d15cdc39e27a4c0a8a55f7e1d46369181307ead9` (both 0.11.2).

The initial feasibility checks exposed `getrandom` target-feature and `libffi-sys` build-script blockers. The later LUNA-03/LUNA-04 patch set target-gated native FFI, C shims, linker options, and platform-only code while retaining native defaults. The pinned parser/core/Xezim checks, release WASM build, real Firefox Worker counter/VCD fixture, and full native Xezim suite then passed. See the [LUNA-04 report](luna-04-virtual-files-report.md) for the reproducible commands, artifact digest, test counts, and current limits.

LUNA-00B built Surfer v0.7.0 from `bd749b1f786c1c62cd67893ca71346cbe6983915` under EUPL-1.2. Its real browser test loaded a counter VCD through same-origin HTTP and Blob URLs, added `top.count`, and moved the cursor to 25 ns. The redistributed WASM SHA-256 is `1341ad01c59f5169209239728417d14f430183abd92515f22bf07cd7c86937ae`. LUNA-05 embedded this bundle in the IDE and passed same-origin VCD load, signal-add, cursor, zoom, and iframe-reload checks in Camoufox Firefox. Upstream marks the inner `InjectMessage` API unstable; keep it isolated behind the versioned adapter. WCP host exports, large-waveform performance, and production Chromium remain unverified.

## Decision

Keep the product boundary as Xezim WASM in a dedicated browser Worker, with no implicit native or JavaScript simulation fallback. The verified Xezim profile is bounded, one-shot RTL simulation with project-relative virtual files and bounded VCD output. Keep native Xezim behavior and filesystem-backed APIs as the default native profile.

Freeze application-facing interface shapes only as described in ADR-002, after review of both feasibility results. Keep concrete wasm-bindgen signatures, Xezim internal file-provider APIs, Surfer inner messages, readiness wire protocol, and waveform URL transport behind their adapters. Do not expose an adapter capability merely because its interface is frozen.

The current worker operation combines compile and run. It does not yet conform to every separate compile/run operation in `SimulationService`. LUNA-10 added a target-specific 256 MiB WebAssembly linear-memory maximum and a 300-second Worker wall-time limit; a per-event or instruction-count budget remains unavailable. Surfer load, signal addition, cursor movement, and zoom are proven only for the pinned test build and tested Firefox environment. `loadWaveform` uses an injected artifact URL resolver and resolves after the wrapper message is posted; the pinned wrapper does not expose an asynchronous waveform-load acknowledgment. WCP remains unavailable.

## Consequences

- Native shared-library DPI/VPI loading, libffi/libloading, JIT/AOT, arbitrary host file access, FST/XTrace, profiling, and pause/resume are outside the browser profile. A later browser-specific extension is verified for scalar `int` DPI imports backed by standalone C/C++-compiled Wasm modules; it does not add the native dynamic-loader path. See the [LUNA-11 DPI/C/C++ report](luna-11-dpi-c-cpp-report.md) and [LUNA-15 MCP end-to-end report](luna-15-dpi-agent-flow-report.md).
- The browser DPI profile is intentionally narrow: imported functions return `int` and accept only scalar `input int` arguments. DPI tasks, output/ref/inout arguments, strings, real values, vectors, aggregates, open arrays, callbacks into SV, and time-suspending calls remain unsupported. Do not infer those capabilities from native Xezim or from the browser C/C++ compiler.
- Includes, `$readmem*`, `$fopen`, and logging have explicit project-virtual-file behavior only for the documented and tested subset. Unsupported tasks must return deliberate diagnostics.
- The browser runtime uses an explicit single-thread entry point, a virtual-file provider, an injected output sink, and a bounded in-memory VCD writer. Native file behavior remains available and covered by the upstream suite.
- Optional simulation operations such as cancellation, current-value lookup, historical transition queries, and unverified viewer controls are not supported by this decision. Add them only with implementations and acceptance tests.
- The interface freeze records boundaries; it does not claim that the complete IDE, MCP bridge, or every adapter is implemented.
