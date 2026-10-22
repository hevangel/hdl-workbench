# `@xezim-ide/protocol`

Versioned wire schemas and application-facing interfaces for the Xezim Studio browser IDE. This package defines protocol `xezim-ide/1`; it does not implement a simulator, storage adapter, MCP transport, or waveform viewer.

Simulation ticks cross JSON boundaries as bounded decimal strings. Convert them to `BigInt` only after parsing; never round them through JavaScript `number`. Browser simulation and Surfer capabilities must remain unavailable until their respective feasibility gates pass.

Build the package and regenerate checked-in Draft 2020-12 JSON Schemas with `pnpm build`. Run contract tests with `pnpm test`.
