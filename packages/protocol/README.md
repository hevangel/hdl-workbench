# `@xezim-ide/protocol`

Versioned wire schemas and application-facing interfaces for the HDL Workbench browser IDE. This package defines protocol `xezim-ide/1`; it does not implement a simulator, storage adapter, MCP transport, or waveform viewer. `simulation.execute` represents the pinned Xezim browser binding's combined compile-and-run Worker call. Separate `simulation.compile` and `simulation.run` requests remain available for implementations that support a compiled design handle.

Simulation ticks cross JSON boundaries as bounded decimal strings. Convert them to `BigInt` only after parsing; never round them through JavaScript `number`. Browser simulation and Surfer capabilities must remain unavailable until their respective feasibility gates pass.

Build the package and regenerate checked-in Draft 2020-12 JSON Schemas with `pnpm build`. See `CHANGELOG.md` for wire changes and the repository ADRs for decisions. Run contract tests with `pnpm test`.
