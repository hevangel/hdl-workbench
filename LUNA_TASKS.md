# Luna Implementation Tasks — Xezim Web IDE

Start with [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md). Each block below can be pasted as a separate assignment to Luna or a coding agent. Agents must work on separate branches/worktrees, test their own changes, and deliver a concise PR report: changed files, commands run, test results, gaps, and follow-on work. Integrator merges against frozen protocol contracts.

**General assignment prefix to give every Luna task:**

> You are implementing a component of the Xezim browser IDE described in IMPLEMENTATION_PLAN.md and AGENTS.md. Read both documents and inspect actual upstream APIs before modifying source. Do not invent existing Xezim/Surfer functions. Create a branch named for your task. Keep changes within assigned paths except when explicitly necessary and documented. Preserve native Xezim behavior. Add regression tests and reproducible build steps. Do not make native simulation a silent fallback for required browser WASM. Commit completed work; report tested vs untested cases.

## Work waves

- **Wave A, immediate parallel:** LUNA-00A WASM feasibility; LUNA-00B Surfer embed spike; LUNA-01 protocol design (can start from plan and adapt after spikes). Each is a separate branch.
- **Wave B, after contracts frozen:** LUNA-02 editor shell, LUNA-03 browser simulator (single owner of Xezim patches), LUNA-05 Surfer adapter, LUNA-06 persistence, LUNA-07 MCP bridge with mocked browser. Separate paths allow parallel work.
- **Wave C, after simulator and adapters:** LUNA-04 source IO/output, LUNA-08 historical wave query, LUNA-09 end-to-end wiring.
- **Wave D:** LUNA-10 hardening/release. This wave may start test design earlier but merges after integration.

## LUNA-00A — WASM feasibility / blocker inventory

**Paths:** `docs/wasm-portability-report.md`, scratch branch, no permanent Xezim refactoring yet.

**Prompt:**

> Pin the current Xezim and xezim-core Git revisions. Inspect Cargo.toml, build.rs, library API, simulation thread model, C/DPI dependencies, and file I/O. Attempt target builds in progression: parser, xezim-core default-features disabled, reduced xezim on wasm32-unknown-unknown. Record exact compiler errors and group by blocking native dependencies vs minor cfg gates. Make a minimal browser worker proof if build permits. Document clear file-level patch plan and risk estimate. Do not claim WASM works based only on `cargo check`. Deliver executable repro commands, the lock SHAs and ADR-001.

**Acceptance:** Reproducible test log, report of blockers with impacted paths, first-pass architecture decision, no regressions.

## LUNA-00B — Surfer browser integration proof

**Paths:** `spikes/surfer-embed/`, `docs/surfer-integration-report.md`.

**Prompt:**

> Pin Surfer WASM assets/version and license. Embed same-origin iframe. Inspect actual pinned `surfer/assets/integration.js`, WCP and `load_url` documentation. Try loading a fixture counter VCD from a Blob URL; prove or disprove with screenshots/test logs. Then prove programmatic add-signal and set-cursor through the *actual supported integration message schema*. If Blob fails, prototype one feasible alternative and document its tradeoffs. Report browser versions and exact asset build SHA. No assumptions about undocumented postMessage shapes.

**Acceptance:** At least one automated browser test that loads a VCD and positions a cursor or an explicit failing test plus a validated fallback decision; adapter spec ready.

## LUNA-01 — Protocol and monorepo foundations

**Paths:** `packages/protocol/`, root `package.json`, `pnpm-workspace.yaml`, `rust-toolchain.toml`, shared CI scaffolding.

**Prompt:**

> Implement monorepo bootstrap using pnpm and a Rust toolchain file. Add shared typed contracts with JSON schemas/Zod for project files, revisions, compile/run commands, diagnostics, waveform artifacts, time ticks/unit, bridge RPC request/results, error codes, and pairing. Include version field `xezim-ide/1`. Write positive/negative validation tests, snapshot provenance tests, and BigInt-safe time serialization tests. Export a WorkspaceAdapter interface and controller interfaces. Freeze contracts in a documented ADR for independent agent lanes.

**Acceptance:** `pnpm install --frozen-lockfile`, `pnpm test`, `pnpm build` work; invalid wire messages rejected; published contract changelog.

## LUNA-02 — Monaco-based browser shell

**Paths:** `apps/ide/src/editor/`, `apps/ide/src/panels/`, app entry/layout, editor UI tests.

**Prompt:**

> Implement browser shell in React/TypeScript/Vite with Monaco SV syntax highlighting, project explorer, tabs, top-module input, Run button, log panel and diagnostics with source location. Use only interfaces from packages/protocol; initially supply mocked WorkspaceAdapter and SimulationService. Add layout persistence hook but not implementation of OPFS. Debounce draft changes and visibly distinguish dirty/saving/saved/save-failed state. Make it keyboard-friendly, responsive and testable. Do not couple UI components to WebSocket or WASM glue directly.

**Acceptance:** Run mock counter with compiler errors and see Monaco markers; tests cover closing tab with unsaved changes and busy controls; build works.

## LUNA-03 — WASM simulator and Worker

**Paths:** `crates/xezim-web/`, `apps/ide/src/workers/`, `vendor/xezim*` or upstream fork patch queue. **Sole owner of Xezim core modifications.**

**Prompt:**

> Implement a true wasm32-unknown-unknown Xezim target for an RTL-only minimal profile. Disable native-only JIT, libffi, libloading, native threads, signal handlers and C shims by compile-time feature/target gates. Retain native builds. Use `wasm-bindgen` to expose a bounded, one-shot `compileAndRun` operation with input source strings and integer-time options, and return structured diagnostics/console/status. Execute it in a dedicated Web Worker. A 4-bit counter must run in a real Chromium browser with NO native simulator/backend. Do not implement fake SV evaluation in TS. Document supported/unsupported features and pin up/downstream patch SHA.

**Acceptance:** Headless real-browser counter completes; Rust native suite passes; reload handles panic/error; interface conforms to protocol. Document stack/memory constraints.

## LUNA-04 — Virtual files and VCD writer

**Paths:** `crates/xezim-web/src/file_provider*`, waveform output adaptation, minimal relevant Xezim file abstraction patch. Coordinate tightly with LUNA-03; don't edit its files concurrently.

**Prompt:**

> Add virtual project-relative file provider for multiple SV files, `include`, `$readmemh`, `$fopen`/read/write subset and associated error diagnostics. Integrate Xezim VcdSink::inline with bounded in-memory writer and proper finalization; capture `$display` and errors. Do not rely on native `std::fs::File` or `stdout` in browser. Establish VCD byte length cap and format/provenance metadata. Test multi-file, nested include, firmware ROM, X/Z, NBA ordering and VCD transition conformance against native Xezim. Unsupported SV system tasks should return deliberate diagnostics, not panic.

**Acceptance:** Browser-generated valid VCD from an SV counter and ROM testbench; native vs WASM expected transitions align; overflow becomes resource-limit error.

## LUNA-05 — Surfer WASM host adapter

**Paths:** `apps/ide/src/surfer/`, pinned `apps/ide/public/surfer/`, tests.

**Prompt:**

> Based on LUNA-00B report, implement SurferAdapter with ready/loadWaveform/addSignals/setCursor/zoom/dispose. Embed a pinned same-origin Surfer WASM iframe. Implement verified postMessage/WCP shape behind adapter, enforce origin/ready state, and do not assume commands accept Blob URL without test. Tests should load a fixture VCD and programmatically add a variable and move the cursor. No direct Surfer private-message calls outside adapter. Include license/asset provenance in public bundle.

**Acceptance:** Browser end-to-end screenshot + commands pass; unknown commands return errors; frame reload reestablishes handshake.

## LUNA-06 — Persistent browser workspace

**Paths:** `apps/ide/src/workspace/OpfsWorkspace*`, metadata database, export/import tests.

**Prompt:**

> Implement WorkspaceAdapter over OPFS for source bytes, IndexedDB for small metadata and editor settings. Use normalized project-relative paths and content revisions; autosave debounce with receipt, restore on reload, explicit Save, quota/out-of-space handling and project ZIP export/import. Request persistent storage where supported but do not treat it as guaranteed. Add a BrowserOnly/Connected workspace authority switch that cannot silently overwrite a Git-backed project. Test reload, multiple tabs, file rename, delete, write conflict and export recovery.

**Acceptance:** Close/reopen tab preserves counter source and project selection; conflict cases never silently discard data; storage errors are shown.

## LUNA-07 — MCP + WebSocket bridge

**Paths:** `services/mcp-bridge/`, MCP contract tests.

**Prompt:**

> Build Node TypeScript bridge with official MCP SDK stdio server and a localhost `ws` listener bound only to 127.0.0.1. Implement pairing with expiring code; check Origin and token on every connection and reject unauthorized requests. Implement typed tools: get_connection_status, list_project_files, read_project_file, write_project_file(expected_revision), compile, run_simulation, get_diagnostics, list_runs, show_waveforms and get_simulation_log. File operations must only access an explicitly selected project root, rejecting absolute paths, traversal and symlink escape. Compile/run forwarded to a mock browser via WS initially. Use request IDs, timeouts, browser-disconnect error and deterministic busy result. MCP protocol stdout must contain only protocol frames; log to stderr. Write Inspector tests and mock browser integration tests.

**Acceptance:** MCP Inspector can invoke tools through bridge; unauthorized webpages and path attacks rejected; disconnect and conflict responses correct.

## LUNA-08 — Historical waveform queries

**Paths:** `packages/wave-index/` or isolated Rust/TS waveform-analysis module; own tests.

**Prompt:**

> Implement deterministic historical waveform queries from VCD artifact, NOT the visual Surfer image. Parse VCD hierarchy/scope/id codes/timescale/four-state vectors and index changes per signal. Expose querySignal(timeTicks) and listTransitions(start,end,pageSize,cursor) with clear unknown signal/out-of-range errors and bounded memory or paging. Compare results with native VCD fixtures including X/Z, aliases, signed vectors and timescale units. Design so OPFS artifacts can be read incrementally later.

**Acceptance:** Query at exact tick matches VCD semantics and returns X/Z precisely; cursor-based pagination works; big signal lists do not transfer in one MCP payload.

## LUNA-09 — Real integration and E2E

**Paths:** `apps/ide/src/controller/`, websocket client, `e2e/`, final integration docs.

**Prompt:**

> Wire WorkspaceAdapter, Monaco, Xezim WASM Worker, waveform artifact store, SurferAdapter and authenticated MCP bridge through one IDE controller. The exact same compile/run controller APIs must serve human buttons and agent commands. Freeze source snapshot and preserve snapshot ID with diagnostics and waveform result. Implement proper busy, timeout, browser-disconnect and stale-revision UX. Write Playwright test: browser opens project; agent connects; modifies SV through MCP; IDE loads new revision; run_simulation forwards into browser; valid VCD loads into Surfer; show_waveforms moves cursor; close/reopen restores files. No native simulator fallback. Add CI E2E.

**Acceptance:** Scripted end-to-end demo green in browser and manual real Codex/MCP smoke green; results reproducible.

## LUNA-10 — Hardening and v1 packaging

**Paths:** `docs/`, `.github/workflows/`, perf fixtures, release scripts.

**Prompt:**

> Audit all deployment paths, origin/pairing safety, write concurrency, quota/memory cap, browser reload and schema compatibility. Keep no browser-only dependency on bridge. Test on Chrome and another engine where supported. Ensure licenses and notices, including Surfer EUPL-1.2, are preserved and distribution implications reviewed. Document unsupported DPI/VPI/JIT/UVM and browser memory constraints. Produce reproducible build and demo scripts, CI matrix, changelog and upgrade notes. Verify native Xezim remains regression-clean.

**Acceptance:** All release checklist items in IMPLEMENTATION_PLAN.md verified; known limitations explicitly listed; shipped assets reproducible and pinned.

## Integrator's sequencing checklist

1. Establish main branch, upstream SHA pins, CI baseline and ADRs.
2. Review LUNA-00A and LUNA-00B **before** over-investing in UI polish.
3. Freeze v1 protocol schemas; require changes to be proposed as ADR instead of private breakage.
4. Start parallel lanes with path ownership and worktrees.
5. Require every PR to include tests and change summary.
6. Merge compiler/Worker before filesystem/output lane to avoid file conflicts.
7. Prove browser-only end-to-end counter before enabling MCP tools.
8. Implement bridge connection and waveform-value query as separate upgrades.
9. Release minimal feature-complete v1 only after data-loss, path security and cross-browser tests.

## What to tell Luna *first*

> Read IMPLEMENTATION_PLAN.md and AGENTS.md. Do not begin a full port. Execute LUNA-00A first. Find the exact upstream revisions, prove `sv-parser` builds to wasm32, inspect `xezim-core` and `xezim` native-only dependencies, and produce a prioritized compile-blocker list with evidence. In parallel, delegate LUNA-00B to a separate branch: embed pinned Surfer WASM, load a fixture VCD, and verify the browser control message contract. Stop and report both feasibility gates before implementing the main IDE.
