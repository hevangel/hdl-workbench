# LUNA-02 — Monaco IDE shell

Date: 2026-10-22
Branch: `codex/luna-02-ui-shell`
Status: automated acceptance checks pass

## Scope delivered

- Added a React 19 and Vite 8 app with a responsive dark IDE layout, project explorer, open-file tabs, Monaco SystemVerilog tokenization, top-module input, Run action, diagnostics and output panels.
- Wired the app to the frozen `WorkspaceAdapter` and `SimulationService` interfaces with explicitly labeled in-memory mocks. The mock run returns `SIMULATOR_UNAVAILABLE`; it does not execute RTL or invoke native Xezim.
- Added debounced draft saves with visible dirty/saving/saved/save-failed states, an unsaved-close decision, busy-state controls, and a localStorage hook for explorer layout preference.
- Configured Monaco to use the pinned local npm package and a bundled editor Worker. The app does not request Monaco from a public CDN.

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
- `pnpm test` — pass, 24 tests total (19 protocol and 5 IDE shell tests).
- `pnpm typecheck` — pass.
- `pnpm build` — pass. Vite emits a 2.96 MB Monaco-containing JavaScript bundle (773 KB gzip) and warns that the chunk exceeds its 500 KB advisory threshold.

The available runner was Node `v25.2.1` with pnpm `11.25.0`; `.node-version` selects Node 24. Node 24 was not installed in this runner.

The UI tests verify Monaco marker calls for a mock diagnostic, close-with-unsaved confirmation and discard, debounced save state, disabled controls while a mock run is busy, and layout preference restoration.

## Unverified and blocked

An in-app browser could not connect to the local Vite server, and a local HTTP request from the runner was also blocked by its socket/network sandbox. The build and component tests pass, but a real-browser visual check of Monaco initialization and the bundled Worker was not possible in this environment.

This milestone does not implement Xezim WASM, persistent project storage, MCP, Surfer, or real compilation/simulation. Xezim and Surfer remain behind the feasibility gates recorded in [ADR-002](ADR-002-interface-contracts.md). The app remains visibly in mock mode until those implementations pass their gates.

## Recommended next task

Proceed with LUNA-03 as the sole owner of Xezim source and feature-gate changes. Keep the editor shell bound to the frozen service interface and leave browser simulation unavailable until the real Worker counter acceptance test passes.
