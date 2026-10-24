# Xezim Web IDE — Implementation Plan

Version 1.0 — 2026-10-21

## Goal

Build an offline-capable browser IDE where a user edits SystemVerilog, Xezim compiles and simulates *inside a dedicated browser Web Worker as WASM*, and embedded Surfer displays the resulting waveform. Preserve projects through browser restarts. Optionally connect a local Codex/Luna agent via a localhost MCP bridge so the agent can edit files, run browser simulation, inspect structured results, and navigate Surfer.

**Hard requirement:** the primary browser simulation path actually executes Xezim WASM. Native Xezim is a differential-test oracle and potential explicitly labeled separate mode, not an invisible substitute.

## Non-goals for the first release

- Arbitrary native DPI-C/VPI plug-ins, Python/cocotb, native Cranelift JIT or AOT.
- A fully interactive pausable event loop, unless a bounded-run API is feasible without compromising scheduling; one-shot runs are sufficient for MVP.
- Full UVM parity, UPF/SDF conformance, multiprocess distributed simulation, collaboration or a hosted backend.
- Guaranteed large-SoC performance or unbounded waveform storage.
- Cloud remote-agent connection directly to a local bridge without an explicit trusted tunnel or agent running locally.

## Verified upstream source observations

- Xezim source: https://github.com/aionhw/xezim (main manifest version seen: 0.11.2; pin exact SHA at implementation kickoff).
- Xezim core: https://github.com/aionhw/xezim-core (default mimalloc global allocator, optional through `default-features = false`).
- Parser crate: `xezim-core/xezim-parser`, Rust-only small dependency footprint.
- Xezim `src/lib.rs` exports `simulate` and `simulate_multi`; the latter normally starts a native thread with a large stack, bypassable when `XEZIM_STACK_MB=0` but still requiring explicit browser-target code.
- `xezim/Cargo.toml` declares libffi/libloading and C shim build; those must be excluded/replaced for browser builds. Native Cranelift is optional.
- Xezim's event loop has a single-thread path; do not introduce browser pthreads for MVP.
- `xezim-core/src/vcd_sink.rs`: `type DumpWriter = Box<dyn Write + Send>` and `VcdSink::inline`, so in-memory VCD sinks are plausible; `xezim/src/compiler/simulator/waveforms.rs` still opens real files and must be adapted.
- File I/O paths such as `src/compiler/simulator/sv_file.rs` need a virtual-file abstraction for `$fopen`, `$readmemh`, etc.
- Surfer: https://gitlab.com/surfer-project/surfer and https://docs.surfer-project.org/book/commands/index.html . Surfer WASM can be embedded, with `postMessage` integration; its low-level integration protocol is unstable. The Waveform Control Protocol (WCP) is separately defined at https://gitlab.com/waveform-control-protocol/wcp. Confirm actual supported browser integration message schema with pinned Surfer assets before implementing the adapter.
- OPFS synchronous file access handles are available within dedicated Web Workers; see MDN `FileSystemSyncAccessHandle` and Origin Private File System docs.
- Codex MCP supports local stdio; official MCP TypeScript SDK available. Reference: https://developers.openai.com/codex/mcp and https://developers.openai.com/plugins/build/mcp-server .

These are source-based observations, NOT claims that the current repositories build for WASM or that Blob URL loading into an iframe already works. Both are integration gates.

## Stack and boundaries

| Area | Choice | Notes |
|---|---|---|
| IDE shell | React + TypeScript + Vite | Pin tested versions in lockfile |
| Editor | Monaco Editor | Register SV language support; diagnostic markers |
| Layout and frontend state | CSS Grid + Zustand | Resizable split, signal selection, logs |
| Schema | JSON Schema + Zod, shared TS package | Versioned operation envelopes; runtime validation |
| Simulation | Rust Xezim + `xezim-core` + `sv-parser` | WASM worker target is `wasm32-unknown-unknown` |
| WASM binding | `wasm-bindgen`, `wasm-pack` | One-shot minimal surface first |
| Threading | Dedicated Web Worker | No native Rust std::thread in browser MVP |
| Waveforms | VCD via in-memory Rust Write sink | FST/streaming as follow-on |
| Surfer | Pinned self-hosted Surfer WASM in iframe | Adapter only; no direct dependencies from IDE state on private Surfer messages |
| Persistence | OPFS content, IndexedDB metadata | Need quota, crash-consistent save, export/import |
| Agent-connected workspace | Local files via Node bridge | Git filesystem is source-of-truth; avoid unsynchronized dual masters |
| MCP bridge | Node.js LTS + `@modelcontextprotocol/sdk` + `ws` | MCP stdio to Codex, localhost WebSocket to browser |
| Validation | Rust native/wasm tests, Vitest, Playwright, MCP Inspector | Golden waveforms plus round-trip tests |
| CI | GitHub Actions | Separate native, WASM, web, integration jobs |

Browser-only mode needs only static assets: web shell, Xezim WASM/JS glue, and pinned Surfer WASM. Bridge is optional. In connected mode, launch the IDE from the bridge’s own loopback origin to simplify origin, WebSocket and waveform URL handling.

## High-level data flow

```text
Codex / Luna
  │ (MCP stdio)
  ▼
Local Node bridge ─────► Approved local Git directory
  │    ▲                            │
  │ WS │                            │ file notifications + revisions
  ▼    │                            ▼
React IDE ◄──────► WorkspaceAdapter / IdeController
  │                               │
  │ editor changes                │ compile/run operations
  ▼                               ▼
Monaco                         Dedicated Web Worker
                                  │
                               Xezim WASM
                                  │
                        VCD bytes / diagnostics
                                  │
                            OPFS + viewer URL
                                  │
                          Surfer iframe (WASM)
```

In browser-only mode, `WorkspaceAdapter = OpfsWorkspace`. In connected mode, `WorkspaceAdapter = BridgeWorkspace` and the approved local project directory is authoritative. Do not mirror edits to both locations as coequal stores; OPFS may hold recoverable unsaved drafts/caches, but writes use revision checks.

## Proposed repository layout

```text
xezim-web-ide/
├─ AGENTS.md
├─ README.md
├─ package.json
├─ pnpm-workspace.yaml
├─ pnpm-lock.yaml
├─ rust-toolchain.toml
├─ apps/
│  └─ ide/
│     ├─ src/
│     │  ├─ editor/                # Monaco + SystemVerilog language setup
│     │  ├─ panels/                # project, logs, diagnostics, waveform
│     │  ├─ controller/            # UI-facing API, worker orchestration
│     │  ├─ workspace/             # OpfsWorkspace, BridgeWorkspace
│     │  ├─ surfer/                # SurferAdapter
│     │  ├─ workers/               # Xezim worker entry
│     │  ├─ bridge/                # WS connection, pairing UI
│     │  └─ app.tsx
│     └─ public/surfer/           # pinned build, LICENSE and provenance
├─ crates/
│  ├─ xezim-web/                  # wasm-bindgen browser-facing API
│  └─ vcd-test-utils/            # optional Rust test helpers
├─ services/
│  └─ mcp-bridge/                # Node stdio MCP server, localhost WS, workspace
├─ packages/
│  └─ protocol/                  # versioned operation schemas, validators, shared types
├─ fixtures/
│  ├─ counter/
│  ├─ multi-file/
│  ├─ rom-readmemh/
│  └─ 4004-smoke/                # later
├─ e2e/                          # Playwright
├─ docs/
│  ├─ ADR-001-runtime-architecture.md
│  ├─ ADR-002-workspace-authority.md
│  ├─ ADR-003-waveform-transport.md
│  └─ compatibility.md
└─ vendor/                       # pinned upstream git revisions or submodules
```

Prefer upstream-friendly feature gates and a small patch set to the existing Xezim repositories. Do not clone an entire second simulator implementation. Maintain an upstream-sha lock and patch queue or a clearly documented fork until modifications can be upstreamed.

## Architecture contracts (freeze before parallel work)

### 1. Workspace

```ts
interface WorkspaceAdapter {
  listFiles(): Promise<FileEntry[]>;
  readFile(path: string): Promise<{ bytes: Uint8Array; revision: string }>;
  writeFile(path: string, bytes: Uint8Array, expectedRevision: string | null): Promise<{ revision: string }>;
  deleteFile(path: string, expectedRevision: string): Promise<void>;
  snapshot(): Promise<{ snapshotId: string; files: SnapshotFile[] }>;
  subscribeChanges(callback: (change: FileChange) => void): () => void;
}
```

Paths are POSIX-like logical project-relative paths. Reject absolute paths, `..` traversal, symlink traversal outside the approved project, and paths not supported by project policy. Revisions are opaque content-hash or monotonic tokens. Agent and human edits must use optimistic concurrency checks. Save should commit to storage before the editor reports “Saved.”

### 2. Browser simulator (proposed)

```ts
interface SimulationService {
  compile(input: CompileRequest): Promise<CompileResult>;
  run(input: RunRequest): Promise<RunResult>;
  getSignalNow?(path: string): Promise<SignalValue>;
  readTransitions?(query: TransitionQuery): Promise<TransitionPage>;
  cancel?(runId: string): Promise<void>;
}
```

`CompileRequest` carries a project snapshot or source file map, explicit top module, include dirs, defines, and mode. `RunRequest` carries a compiled snapshot ID, max *simulation time as an integer tick with an explicit time unit/timescale*, max work budget and optional VCD scope selection. Output includes `runId`, `snapshotId`, diagnostics, stdout, status (`completed`, `finish`, `failed`, `timeout`, `resource_limit`), last simulation tick, waveform artifact ID/size. `getSignalNow` is optional MVP. Historical `readTransitions` requires waveform indexing; DO NOT fake it using Surfer screenshots or claim it exists until implemented.

### 3. MCP-to-browser RPC

```json
{
  "protocol": "xezim-ide/1",
  "id": "request-uuid",
  "op": "simulation.run",
  "projectId": "active-project-id",
  "params": { "snapshotId": "sha256:...", "until": { "ticks": "1000000", "unit": "ps" } }
}
```

Successful response:

```json
{
  "protocol": "xezim-ide/1",
  "id": "request-uuid",
  "ok": true,
  "result": { "runId": "run-42", "status": "finish", "waveformArtifactId": "wave-42" }
}
```

Failed response:

```json
{
  "protocol": "xezim-ide/1",
  "id": "request-uuid",
  "ok": false,
  "error": { "code": "BROWSER_DISCONNECTED", "message": "No paired IDE session" }
}
```

Use JSON for control; do NOT inline multi-megabyte waveforms in MCP JSON. Binary transfers use separate chunked channel/OPFS artifacts (and can have a content-addressed local HTTP endpoint when bridge is active). Each message has a monotonic request ID and timeouts. The bridge enforces only one active compile/run per connected browser simulator, allowing queued read requests where safe. Responses include snapshot provenance.

### 4. Waveform artifact

```ts
type WaveformArtifact = {
  id: string;
  runId: string;
  snapshotId: string;
  format: 'vcd' | 'fst';
  sizeBytes: number;
  storage: 'opfs' | 'bridge-file';
  timescale: { magnitude: string; unit: 's'|'ms'|'us'|'ns'|'ps'|'fs' };
  complete: boolean;
};
```

Never mix ns and raw simulator ticks. Use integers/stringified large integers end-to-end (BigInt in JS where appropriate). Preserve `x`/`z` separately from numeric value.

## Detailed integration design

### A. Xezim WASM

1. **Compile gate:** `cargo check -p sv-parser --target wasm32-unknown-unknown`, next `xezim-core` with default allocator off, then a reduced Xezim crate. Capture and classify all compile errors. Stop and revise estimate if pervasive native dependencies are found.
2. Add feature `browser` and/or `cfg(target_arch = "wasm32")` gates so `libloading`, `libffi`, C shims, native JIT, signal handlers, `std::thread::spawn`, `/proc` sampling and OS/terminal helpers are absent on browser target. Preserve native builds and tests. The exact dependency graph determines how the target-specific Cargo sections are implemented.
3. Expose `WebSimulator::compile()` / `run()` through `wasm-bindgen`; start one-shot, bounded runs on dedicated browser Worker. Existing Xezim `simulate_multi()` uses worker threads by default; refactor or call a safe single-thread path without environment-variable hacks when targeting browser.
4. Remove synchronous native filesystem assumptions from parser include-loading, `$readmemh`, `$fopen`, `$fwrite`, and dump creation by defining project-relative `VirtualFileProvider`. For MVP preload inputs into the Worker before starting simulation. Output writes should go to a bounded byte sink.
5. Route `$display` and diagnostics to an injected capture sink, not native stdout. Failure/panic must become structured error where possible. Do not rely on Rust panic unwinding from `panic=abort` builds.
6. Reuse `VcdSink::inline(Box<dyn Write + Send>)` with a bounded in-memory writer. Instrument byte limits and finalize before handing data to JS. `fst-writer` is a later target.
7. Compile smoke tests: counter, x/z, NBA ordering, nested include, `$readmemh`, `$finish`, `$fatal`, `$dumpvars`. Compare native Xezim behavior and WASM output. Pin source SHA and fixtures.
8. A long blocking synchronous WASM call prevents the same Worker from processing ordinary cancellation messages. For MVP, cap simulation time/work and optionally terminate/recreate the Worker (loses state). Real pause/resume/interrupt requires a resumable event loop with safe yield points, or a carefully validated atomic flag mechanism. Do not promise responsive pause in v1.

### B. Editor and frontend

- Left: project tree, top-module picker. Center: Monaco file tabs. Bottom: build output, diagnostics, regression summary. Right/bottom: Surfer iframe.
- Ship basic SV language grammar and syntax highlighting first; do not claim a full LSP unless it is built. Map diagnostics to `(path, line, column, severity, message)`.
- On change, mark draft revision dirty and debounced autosave with explicit saved/unsaved/failed states. Project snapshot for compile must reflect the currently visible editor contents, not stale saved files. For local-backed projects, await successful save before compiling or require an explicit intentional in-memory snapshot.
- On Run, disable conflicting actions, pass input snapshot to Worker and show progress. Return result to controller, store waveform artifact, then request Surfer load.
- Display resource limits and error state when browser memory/quota is exhausted.

### C. Surfer integration

1. Pin Surfer version and license assets. Self-host under `/surfer/` to avoid cross-origin postMessage confusion.
2. Implement `SurferAdapter` API: `ready()`, `loadWaveform(artifact)`, `addSignals(paths)`, `setCursor(time)`, `zoom(start,end)`, `dispose()`.
3. Study pinned build `surfer/assets/integration.js` and WCP support; implement only the verified message schema. Use allowed origin and readiness handshake, not wildcard trust.
4. **Integration gate:** test whether a generated `blob:` URL will load via Surfer's browser `load_url` command within a same-origin iframe. The VCD extension/filename and fetch behavior are uncertain; do not assume.
5. If blob fails, choose a documented fallback: (i) a verified Surfer byte-loading host integration; (ii) a Service Worker same-origin route to stored waveform bytes; or (iii) in bridge mode a safe loopback `/artifacts/<id>.vcd` endpoint. Confirm feasibility with browser integration test. Avoid modifying Surfer until a narrow adapter approach is exhausted.
6. Keep waveform analysis separate from the viewer. Initial `show_waveforms` only navigates visualization; historical value queries require an indexed reader in a later milestone. Use Surfer commands `variable_add`, `cursor_set`, `zoom_to` if their access through the pinned integration is verified.

### D. Browser persistence

- `OpfsWorkspace` stores file bytes under `/projects/<id>/...`; IndexedDB stores project list, selected tabs, UI layout, source revisions, waveform metadata. Use safe/migratable schema versions.
- Autosave after a short debounce; flush on explicit Save and after critical transitions. `beforeunload` is best effort, not reliable; never use it as the sole persistence mechanism. On restart recover last confirmed save and, if necessary, restore an uncommitted draft.
- Store waveforms separately from source; give user delete/export options. Quota errors must not overwrite source data.
- On supported browsers offer opening/saving a selected real folder. Provide ZIP project import/export as a universal fallback. Browser storage is not permanent backup; support export and offer `navigator.storage.persist()` without promising success.
- For large waveform work, write chunks into OPFS in a worker using `FileSystemSyncAccessHandle`, respecting exclusive locks and flush semantics.

### E. MCP bridge

- Single Node process started by Codex MCP `stdio` registration and listening on `127.0.0.1:8765` WebSocket, with configurable port. **Do not write diagnostic logs to stdout**, which is reserved for MCP JSON-RPC; stderr only.
- For local dev, host UI through the bridge's loopback origin or Vite proxy; for standalone HTTPS deploy, separately test mixed-content/Private Network Access behavior before declaring remote page + localhost bridge supported.
- Pairing: MCP tool `get_connection_status` reports a one-time short pairing code; human enters it into webpage to bind the current browser session. Use short expiry, cryptographically random token, origin allowlist, WebSocket authorization, loopback bind only, explicit disconnect. Browser session can authorize read-only vs write access.
- MCP commands initially: `get_connection_status`, `list_project_files`, `read_project_file`, `write_project_file(expected_revision)`, `compile`, `run_simulation`, `get_diagnostics`, `list_runs`, `show_waveforms`, `get_simulation_log`.
- Later: `query_signal_at_time` and `query_signal_transitions` after waveform reader/index exists; `pause`, `resume`, `step` only after resumable scheduler exists. Every tool needs JSON input/output schemas and clear errors.
- `write_project_file`: local bridge persists a validated write into approved workspace, notifies the connected IDE, receives revision acknowledgments, and refuses write conflict. Editing from web uses same bridge filesystem when connected. Use one authoritative store.
- `compile` and `run_simulation`: forward op to browser IdeController via WS. Return bounded diagnostic summaries and artifact refs, not huge waveform dumps. If disconnected, fail explicitly. No implicit fallback to native Xezim.
- Restrict project paths to approved root (including symlink resolution), set file size limits, avoid arbitrary shell or process execution tools, reject or ask for approval on destructive operations. Browser source cannot access native host via DPI; native DPI and `$system` remain disabled.
- Use MCP Inspector to test tool discovery and read/write/reconnect/error paths; Codex CLI config path per official docs.

### F. Native and browser source reconciliation

Two distinct modes:

- **Browser-only:** OPFS is source of truth; the browser can export source as ZIP or save via user-selected directory.
- **Agent-connected:** local Git directory is source of truth through bridge; OPFS may hold drafts and waveform cache but is NOT automatically another master. Source files carry revisions. On unexpected change, present 3-way conflict; never overwrite unsaved edits.

`compile` freezes a `snapshotId` from all active project files and run options. The result references that snapshot. Subsequent edits make displayed results “based on older revision,” not silently attached to current code.

## Security, safety and licensing

- Local bridge binds only to loopback, requires pairing, validates origin and schema, and restricts paths to an approved directory. Treat a local WebSocket without authentication as vulnerable to websites that attempt to connect to localhost.
- Use request correlation IDs, timeouts, heartbeats and explicit `BROWSER_DISCONNECTED`, `REVISION_CONFLICT`, `INVALID_PATH`, `SIMULATION_BUSY`, `QUOTA_EXCEEDED` errors.
- Harden untrusted RTL input and waveform parsers with size, runtime and allocation caps. The Worker provides isolation from UI hangs, not a universal sandbox guarantee.
- Do not enable native code execution or dynamic FFI on the browser path.
- Preserve upstream license notices. Surfer uses EUPL-1.2; Xezim source manifests identify MIT OR Apache-2.0. Have the distribution/compliance plan reviewed for self-hosted Surfer assets and any modifications. Do not assume all license combinations are interchangeable.
- Do not assume total system snapshots or `get_signal(time)` until corresponding capabilities are implemented.

## Milestone plan and dependency graph

| ID | Milestone | Acceptance gate | Dependencies | Estimate |
|---|---|---|---|---|
| M0 | Research/spikes | Pin revisions; wasm parser check; Surfer embed + sample VCD proof; decision log | none | 3–5 working days |
| M1 | Contract and shell | Monorepo builds; protocol schemas; mocked compile/run; Monaco editor; test harness | M0 for final choices | 3–5 days |
| M2 | WASM core | Xezim counter compiles/runs in real browser Worker; native differential test | M0 | 1–3 weeks, high uncertainty |
| M3 | VFS and wave output | nested include, `$readmemh`, `$display`, valid VCD and output cap | M2 | 1–2 weeks |
| M4 | Surfer | Generated VCD opens in pinned iframe; programmatically navigate waveforms | M0, M3 | 3–7 days |
| M5 | Persistence | Autosave/reload, recovery, export, quota handling | M1 | 3–7 days |
| M6 | MCP bridge | Local agent pairs, edits, compiles and views results, no native fallback | M1, M2, M4, M5 | 1–2 weeks |
| M7 | Stabilization | Browser E2E, CI matrix, failure safety, documentation | M3–M6 | 1–2 weeks |

Estimates are engineering planning ranges, not commitments. M2 is the main schedule uncertainty: native-only subgraphs may demand more refactoring than expected. With sensible parallelism, a limited end-to-end prototype may be feasible in several weeks; hardened full v1 will take longer.

Dependency graph:

```text
M0 ─┬─ M1 ─── M5 ───┐
    ├─ M2 ── M3 ── M4 ├─ M6 ── M7
    └─ Surfer spike ─┘
```

## Work allocation for Luna (merge as independent PRs)

| Task | Owner lane | Files owned | Prereqs | Done when |
|---|---|---|---|---|
| LUNA-00 | Tech lead/research | `docs/ADR-*`, `docs/compatibility.md` | none | Pinned upstream SHAs, WASM error classification and Surfer embed spike captured |
| LUNA-01 | Protocol | `packages/protocol/` | LUNA-00 design | Zod schemas, tests and generated types for requests/responses |
| LUNA-02 | Web shell/editor | `apps/ide/src/editor`, `panels`, app entry | LUNA-01 stub schemas | Monaco, file tabs, diagnostics, mock Run UI |
| LUNA-03 | WASM portability | `crates/xezim-web/`, minimum patches in `vendor/xezim*` | LUNA-00 | Counter sim in real browser Worker, no native backend |
| LUNA-04 | Virtual FS/output | WASM file provider and VCD sinks only | LUNA-03 | Multi-file include, ROM hex, console, VCD oracle tests |
| LUNA-05 | Surfer adapter | `apps/ide/src/surfer`, `public/surfer` | LUNA-00 (can use fixture VCD first) | Pinned viewer opens fixture/generated VCD and receives cursor/signal commands |
| LUNA-06 | Persistence | `apps/ide/src/workspace/` | LUNA-01 | Autosave/restart, file revisions, export/import, quota tests |
| LUNA-07 | Bridge/MCP | `services/mcp-bridge/` | LUNA-01; mock browser | Pairing + stdio tools + authenticated WS; real-browser integration in LUNA-09 |
| LUNA-08 | Waveform query | `packages/wave-index/` or isolated module | LUNA-04 | Historical four-state value and transition pagination with tests |
| LUNA-09 | End-to-end orchestration | `apps/ide/src/controller/`, `e2e/` | LUNA-02–07 | Codex edit→compile→run→viewer path works and survives restart |
| LUNA-10 | Hardening & release | CI, docs, security tests, performance | LUNA-09 | Release checklist green |

Good concurrency: LUNA-01 and M0 spikes, then LUNA-02/03/05/06/07 in separate branches after schema freezes. Avoid having two agents edit Xezim's `src/lib.rs`, manifest or simulator setup simultaneously; put all native-core changes in one lane. Integrator reviews interfaces between every merge.

## Proposed tasks in deliverable order

**First demonstration:** From a local web page, paste a small counter SV testbench, click Run, and see Xezim-WASM-generated VCD in Surfer; no backend.

**Second demonstration:** Edit file, close tab, reopen and recover exact contents. Run after reopen.

**Third demonstration:** `codex mcp` calls a local bridge tool to edit the project, compile and simulate in the browser, then place Surfer cursor on a failing interval; simulate when browser disconnected must return a clear error.

**Fourth demonstration:** Query precise historical X/Z values (requires waveform indexing), support bigger persistent waveform artifacts, and advanced pause/resume.

## Minimum golden fixtures

- `counter`: 4-bit clk/reset counter, `$finish`, `$display`, `$dumpvars`; same final values/time and VCD transitions native vs WASM.
- `nba_ordering`: blocking/nonblocking assignment test covering scheduling correctness.
- `four_state`: x/z transitions, 16/64/128-bit vectors, signed/unsigned arithmetic, no accidental two-state conversion.
- `multi_file`: include, timescale, nested dirs, macro defines.
- `rom`: `$readmemh` on logical `firmware.hex`, correct byte ordering and path errors.
- `negative`: syntax error + line/column, missing include, invalid top, zero-time oscillation/work cap, `$fatal` handling.
- `storage`: save 10 files, reload, conflicts, quota failure, deletion and export/import recovery.
- `mcp`: pair/disconnect, cross-origin rejection, path traversal rejection, duplicate requests, concurrent run busy, stale snapshot checks.

## Quality gates

- Tests run: `cargo test` native, `cargo check --target wasm32-unknown-unknown`, wasm build; `pnpm test`, `pnpm build`, `pnpm test:e2e`; MCP Inspector tool contract suite.
- End-to-end Xezim simulator must run without a native server in browser-only mode.
- No silent data loss on normal reload, no successful Save indicator before durable store completes, no overwrite of conflicting revisions.
- Browser Worker prevents page freeze for a bounded finite simulation; runaway simulations cannot allocate indefinitely.
- All agent tool actions log request ID, project ID, snapshot, operation and status without logging sensitive file contents by default.
- `X/Z` are represented correctly in waveform and agent API; integer time units do not drift.

## Release checklist

- [ ] Upstream SHAs, dependency versions, patch licenses and Surfer assets pinned
- [ ] Browser-only counter and ROM demonstration
- [ ] Bounded waveform output and memory guard
- [ ] Autosave and export verified on browser restart
- [ ] Surfer control no reliance on unverified message payloads
- [ ] MCP schema, pairing, origin and path authorization tests
- [ ] Browser disconnected/reconnected and revision conflict tests
- [ ] CI green on Chrome + at least one additional target browser where feasible
- [ ] User-facing unsupported capabilities documented
- [ ] Native Xezim regressions unchanged

## Resource and performance planning

Start with conservative configurable caps (illustrative defaults to be tuned by measured browser tests): maximum project input bytes 16 MiB, generated in-memory VCD 64 MiB, bounded simulation-time/work budget, and one simulator Worker per active project. Large output should switch to chunked persistent OPFS rather than holding multiple complete buffers. Export individual files and project ZIP. Surface estimated storage usage with `navigator.storage.estimate()`; ask for persistent storage with `navigator.storage.persist()` but do not promise approval.

`wasm32-unknown-unknown` predominantly uses 32-bit linear-memory addresses, and real available memory depends on browser/device; do not size for large SoCs without benchmarks. Pin actual browser tests, track source snapshot and waveform hashes. Favor deterministic regression fixtures over performance claims.

## Risks and decisions

1. **Highest: wasm-target compilation of simulator.** Mitigation: fail-fast parser/core/VM spike; target-specific feature gates, scope RTL-only first.
2. **Surfer host-to-iframe waveform load.** Mitigation: fixture VCD integration spike and adapter; verify `blob:` before choosing loader.
3. **File I/O deep inside runtime.** Mitigation: `VirtualFileProvider` abstraction, narrow caller patching, explicit unsupported operations.
4. **Worker cancellation responsiveness.** Mitigation: bounded initial run and `terminate()`; resumable tick/event loop later.
5. **Historical waveform data API.** Mitigation: separate waveform index from Surfer rendering; do not infer data visually.
6. **Data loss and conflicting edits.** Mitigation: one authority per mode, revision checks and autosave receipts.
7. **Local MCP security.** Mitigation: authenticated pairing, loopback bind, origin validation and approved root.
8. **Cross-browser file/directories, mixed-content/private-network differences.** Mitigation: OPFS and ZIP baseline, Chromium-first local agent mode, compatibility matrix.
9. **License and distribution.** Mitigation: preserve Xezim and Surfer notices and review EUPL implications before embedding a modified Surfer build.

## Sources

- Xezim: https://github.com/aionhw/xezim
- Xezim-core: https://github.com/aionhw/xezim-core
- Surfer integration and license: https://gitlab.com/surfer-project/surfer
- Surfer commands: https://docs.surfer-project.org/book/commands/index.html
- WCP specification: https://gitlab.com/waveform-control-protocol/wcp
- Codex MCP: https://developers.openai.com/codex/mcp
- MCP TypeScript SDK guidance: https://developers.openai.com/plugins/build/mcp-server
- OPFS: https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system
- OPFS sync handles: https://developer.mozilla.org/en-US/docs/Web/API/FileSystemSyncAccessHandle
