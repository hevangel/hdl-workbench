# LUNA-01 — Protocol and monorepo foundation

Date: 2026-10-22
Branch: `codex/luna-01-protocol-foundation`
Status: acceptance checks pass

## Scope delivered

- Added the pnpm workspace root, pinned package manager and JavaScript dependencies, Node version file, and Rust toolchain file.
- Added `@xezim-ide/protocol` with strict Zod v4 schemas and checked-in Draft 2020-12 JSON Schema output for protocol `xezim-ide/1`.
- Defined project paths/files/revisions/snapshots, compile and run requests/results, diagnostics, bounded work budgets, VCD artifact metadata, tick values and timescales, pairing messages, capability status, RPC requests/results/errors, and WorkspaceAdapter, SimulationService, SurferAdapter, and IdeController interfaces.
- Added RPC request/result correlation and snapshot provenance validation. Tick values cross JSON as canonical decimal strings and compare through `BigInt` after validation.
- Documented the package as the wire contract source of truth in [ADR-002](ADR-002-interface-contracts.md) and added the package changelog.

## Reproducible commands and results

Run from the repository root with Node 24 and pnpm 11.25.0:

```powershell
pnpm install --frozen-lockfile
pnpm test
pnpm typecheck
pnpm build
```

Results on this checkout:

- `pnpm install --frozen-lockfile` — pass.
- `pnpm test` — pass, 19 tests.
- `pnpm typecheck` — pass.
- `pnpm build` — pass; regenerates the checked-in JSON schemas.

The available runner was Node `v25.2.1` with pnpm `11.25.0`; the repository pins the recommended Node major through `.node-version` as `24`. Node 24 was not installed in this runner, so that exact runtime remains unverified here.

The test suite covers accepted and rejected project paths, strict/versioned RPC envelopes, unsupported operations, pairing-code validation, canonical decimal tick validation, project/snapshot provenance, response id/op/result matching, and JSON round-tripping above `Number.MAX_SAFE_INTEGER`.

## Findings and limits

The v1 contract is implemented independently of Xezim and Surfer internals. Unknown object fields are rejected, and every successful RPC result must match one of the known result schemas. `validateRpcResponseForRequest` further checks the result shape for its originating operation and verifies compile/run snapshot provenance. No `cancel`, `getSignalNow`, or `readTransitions` API was added.

The dependency pins are Zod `4.6.7`, TypeScript `7.0.2`, and Vitest `5.0.5`; pnpm is pinned to `11.25.0`. No simulator, persistent workspace, UI, MCP server, or waveform viewer is claimed as implemented by this contract package.

The inherited feasibility blockers remain open:

- **Xezim:** the baseline WASM checks fail through native-only dependencies, and the reduced Xezim target currently reaches a `libffi-sys` Autoconf failure on `wasm32-unknown-unknown`. See [the build-gate follow-up](wasm-build-gate-followup.md). Browser simulation remains unavailable; native Xezim is not a fallback.
- **Surfer:** the pinned viewer source did not provide a reproducibly pinned browser artifact and its actual load/add-signal/cursor browser behavior remains unverified. See [the Surfer report](surfer-integration-report.md). Viewer capabilities remain unavailable.

## Recommended next tasks

1. Start LUNA-02 using only `@xezim-ide/protocol` interfaces and mocked adapters to build the Monaco editor shell.
2. Start LUNA-03 as the sole owner of Xezim source and native-preserving target gates; keep the browser simulator capability disabled until the real Worker counter acceptance test passes.
3. Keep LUNA-05 behind the real-browser Surfer artifact/API gate. Do not implement guessed private-message payloads.
