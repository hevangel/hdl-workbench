# AGENTS.md — using and changing HDL Workbench

HDL Workbench is a browser SystemVerilog workspace. The person's browser runs the Xezim simulator as WebAssembly in a Web Worker and shows waveforms in an embedded Surfer viewer. You, the agent, reach it through a local MCP stdio server (`services/mcp-bridge`) that owns a project folder on disk and forwards simulation requests to the paired browser tab.

Part 1 is how to use the workbench to write and simulate HDL. Part 2 is the rules for changing this repository. Human-oriented setup is in [`README.md`](README.md); the visual architecture overview is [`docs/index.html`](docs/index.html).

---

## Part 1 — Driving the workbench over MCP

### Mental model

```text
you (MCP client) ──stdio──► mcp-bridge ──ws://127.0.0.1──► browser tab ──► Xezim WASM Worker
                               │                                 │
                        project root on disk                OPFS VCD ──► Surfer
```

- **The disk project is the source of truth** while the browser is paired. You edit files on disk through the bridge; before every run the bridge pushes an immutable, hashed snapshot to the browser.
- **The bridge never simulates.** Every run happens in the person's browser. If no browser is paired, simulation tools fail with `BROWSER_DISCONNECTED`; nothing falls back to a native or JavaScript simulator. Never invent results, and never run native `xezim` and present its output as workbench output.
- **The person is watching.** Runs, logs and waveforms appear in their tab. Keep actions purposeful.

### 1. Start and pair

**Shell-only agents** (no MCP server configured): use the `hdlwb` session CLI and follow the skill [`.agents/skills/hdl-workbench-cli/SKILL.md`](.agents/skills/hdl-workbench-cli/SKILL.md). It keeps one paired bridge alive across shell commands and exposes the same tools listed below.

The MCP server is normally configured by the person. If you must start it, use:

```sh
pnpm --filter @xezim-ide/mcp-bridge build
node services/mcp-bridge/dist/main.js --project-root <absolute project folder>
```

Options are `--port` (default `47531`; `0` picks a free port), `--allowed-origins` (default `http://localhost:5173,http://127.0.0.1:5173`; it must include the exact IDE origin), and `--request-timeout-ms` (default 30000). Each has an environment-variable form: `XEZIM_PROJECT_ROOT`, `XEZIM_BRIDGE_PORT`, `XEZIM_ALLOWED_ORIGINS` and `XEZIM_BRIDGE_TIMEOUT_MS`. Stdout belongs to MCP, and diagnostics go to stderr. The IDE must already be running (`pnpm demo`) for the browser to pair.

Pairing:

1. Call `get_connection_status`. If `connected` is `true`, skip to step 3.
2. Otherwise give the person the **entire JSON result** and ask them to click **Connect agent** in the IDE, paste it, and press **Pair browser**. The code is single-use and expires in two minutes, so call `get_connection_status` again for a fresh one if pairing fails. Don't repeat the pairing code anywhere else.
3. Confirm with `get_connection_status` that `connected: true` before simulating.

### 2. Tool reference

All paths are project-relative POSIX paths (`rtl/top.sv`). Absolute paths, `..`, and symlinks that escape the root are rejected with `INVALID_PATH`. Every result is JSON in a text content block; failures set `isError` and contain `{ code, message }`.

| Tool | Input | Use |
| --- | --- | --- |
| `get_connection_status` | `{}` | Pairing info and whether a browser is connected |
| `list_project_files` | `{}` | Every file with its `revision` (`sha256:…`) |
| `read_project_file` | `{ path }` | Content (base64) and current `revision` |
| `write_project_file` | `{ path, expected_revision, content_base64 }` | Create (`expected_revision: null`) or replace (current revision). A stale revision returns `REVISION_CONFLICT`. |
| `run_simulation` | `{ top_module, until_ticks, timescale, work_budget?, plusargs?, waveform_format? }` | Sync the snapshot, then compile and run it in the browser. Returns a `RunResult`. |
| `get_simulation_log` | `{ run_id, tail_bytes? }` | Captured stdout/stderr (default tail 64 KiB, max 1 MiB) |
| `get_diagnostics` | `{ run_id? }` | Structured diagnostics with file, line and code |
| `list_runs` | `{ limit? }` | Recent runs (default 20, max 100) |
| `show_waveforms` | `{ artifact_id, signal_paths?, cursor? }` | Open a run's VCD in Surfer, add signals, place the cursor |
| `compile` | `{ top_module, … }` | **Not available in the browser.** It returns `UNSUPPORTED_CAPABILITY`, so use `run_simulation`. |

`run_simulation` details:

- `until_ticks` is a **decimal string** (`"1000"`), not a number. With `timescale` `{ "magnitude": "1", "unit": "ns" }` it means 1000 ns. The total must be a whole number of nanoseconds and at most 10,000,000 ns.
- `work_budget` defaults to `{ max_wall_time_ms: 30000, max_output_bytes: 1048576, max_vcd_bytes: 16777216 }`. The maximums are 300000 ms, 16 MiB and 64 MiB.
- `plusargs` is an optional array of at most 64 strings such as `"+verbose"` or `"+N=42"`. The simulator reads them with `$test$plusargs` and `$value$plusargs`. Each must start with `+` and contain no spaces.
- `waveform_format` is `"vcd"` (default) or `"fst"`. FST is converted from the simulator's VCD in the browser. Either way, `waveform` in the result points to the stored artifact.
- The result contains `runId`, `snapshotId`, `status` (`completed`, `finish`, `failed`, `timeout` or `resource_limit`), `diagnostics`, `stdout`, `stderr`, `lastTime`, and `waveform` when a VCD was produced. Pass `waveform.id` to `show_waveforms` as `artifact_id`.
- Only one simulation runs at a time; an overlapping call returns `SIMULATION_BUSY`.

### 3. Standard loop

1. `list_project_files` → note revisions.
2. `read_project_file` for anything you will change; edit; `write_project_file` with the revision you just read. On `REVISION_CONFLICT`, re-read and merge, because the person may have edited the file. Never blind-overwrite.
3. `run_simulation` with an explicit `top_module` and a stop time that covers the test.
4. Check `status` **and** `stdout`. `finish` means `$finish` ran. `completed` means the stop time was reached. A test only passes when its own pass/fail output says so.
5. On failure, read `diagnostics` (or `get_diagnostics`) and `get_simulation_log`, fix, and rerun.
6. When a waveform helps, call `show_waveforms` with the VCD hierarchy paths you care about (`top.dut.count`) and an optional `cursor: { ticks }` in the VCD's timescale.
7. Report what you ran: `top_module`, stop time, `status`, the relevant output lines, and `runId`.

### 4. Writing HDL that works here

- Give testbenches a `` `timescale `` and end them with `$finish`.
- For a waveform, add `$dumpfile("<name>.vcd"); $dumpvars(0, <top>);`. Without them, no VCD artifact is produced.
- `$readmemh`, `$readmemb`, `$fopen` and `$display` work against **project-relative** files from the snapshot. Host paths outside the project do not exist.
- Make self-checking tests print one clear marker, such as `PASS` or `FAIL: <reason>`, so the result can be read from `stdout`.
- **DPI (browser profile):** put C/C++ next to the SV (`.c`, `.cc`, `.cpp`, `.cxx`, plus headers). Only `import "DPI-C" function int f(input int a, …)` is supported, with an `int` return and scalar `input int` arguments only. C++ needs `extern "C"`. libc/libc++ calls, exceptions, RTTI, cross-file unresolved symbols, DPI tasks, `output`/`ref` arguments, strings, arrays, callbacks into SV and exported SV functions are unsupported. A C/C++ compile failure returns `DPI_COMPILATION_FAILED`.
- Unsupported features include native DPI/VPI `.so`/`.dll`, Python/cocotb, full UVM, FSDB, pause/resume or stepping, and in-run cancellation. Command-line flags other than `+plusargs` and `-sv_seed` do not exist in the browser; FST is produced by converting the simulator's VCD in the browser (`crates/xezim-web/src/fst.rs`), not by Xezim's native FST writer. Don't suggest that they work, and don't infer browser support from native Xezim.

### 5. Errors and what to do

| Code | Meaning → action |
| --- | --- |
| `BROWSER_DISCONNECTED` | No paired tab. Repeat the pairing steps; ask the person to keep the IDE tab open. |
| `REVISION_CONFLICT` | The file changed since you read it. Re-read, merge, and write with the new revision. |
| `INVALID_PATH` | Path is absolute, traverses out, or escapes the root. Use a project-relative path. |
| `SIMULATION_BUSY` | Another run is in progress. Wait for it, then retry once. |
| `TIMEOUT` | Wall-time budget or bridge request timeout hit. Shorten the stop time, fix a hang (missing `$finish`, zero-delay loop), or raise `max_wall_time_ms` within 300000. |
| `RESOURCE_LIMIT` | 256 MiB WASM memory, output, VCD or input cap. Reduce the design, dump fewer scopes, or shorten the run. |
| `SIMULATOR_UNAVAILABLE` | The IDE was started without built WASM assets. Ask the person to run `pnpm build:browser` (or `pnpm demo`). Do not work around it natively. |
| `UNSUPPORTED_CAPABILITY` | A deliberately absent feature, such as `compile`. Use the supported path. |
| `DPI_COMPILATION_FAILED` | Fix the C/C++ source shown in the diagnostics. |
| `QUOTA_EXCEEDED` | Browser storage is full. Ask the person to export or remove old projects. |
| `UNAUTHORIZED`, `INVALID_REQUEST` | Session or schema problem. Re-pair or correct the input shape. |

Limits worth remembering: 4,096 files and 16 MiB of source per browser run (the bridge snapshot cap is 64 MiB with 16 MiB per file); VCD up to 64 MiB, with the browser keeping 8 VCDs and 128 MiB in total.

---

## Part 2 — Changing this repository

### Map

| Path | Owns |
| --- | --- |
| `apps/ide/` | React IDE, `BrowserIdeController`, simulator Worker (`workers/xezim.worker.ts`), DPI compiler Worker, OPFS workspace, Surfer adapter (`surfer/SurferIframeAdapter.ts`) |
| `crates/xezim-web/` | wasm-bindgen wrapper: `compileAndRunProject` and bounded output/VCD |
| `packages/protocol/` | `xezim-ide/1` Zod schemas, generated JSON Schema, shared path ordering |
| `packages/wave-index/` | VCD parser and historical signal queries |
| `services/mcp-bridge/` | MCP tools (`src/mcp.ts`), service logic, project-file authority, WebSocket pairing |
| `vendor/xezim`, `vendor/xezim-core` | Pinned upstream sources with documented, target-gated browser patches ([`vendor/UPSTREAM.md`](vendor/UPSTREAM.md)) |
| `services/mcp-bridge/scripts/hdlwb.mjs` | Session CLI that keeps one paired bridge alive for shell-only agents |
| `.agents/skills/` | Agent skills, such as `hdl-workbench-cli` for driving the web app through `hdlwb` |
| `docs/index.html` | Public visual overview, which must stay accurate to the code |
| `session_history/initial_build/` | Historical ADRs, plan, task lanes and milestone reports. These are records: add new reports beside them rather than rewriting old results. |

### Invariants

- **Browser simulation is Xezim WASM in a Worker.** Never add a native, server or JavaScript fallback, and never fake a capability. Report it as unavailable instead.
- **Truthful capabilities.** A feature exists only when code and tests prove it. Unimplemented contract operations return `UNSUPPORTED_CAPABILITY` or `SIMULATOR_UNAVAILABLE`. Don't claim pause/resume, `getSignalNow`, historical queries from Surfer pixels, FSDB, or broader DPI.
- **Typed boundaries.** The IDE controller, WorkspaceAdapter, simulation Worker, SurferAdapter and MCP bridge talk only through `packages/protocol` contracts. Wire changes need a protocol `CHANGELOG.md` entry and regenerated schemas (`pnpm build`). Surfer's unstable `InjectMessage` payloads stay inside `SurferIframeAdapter.ts`.
- **Data safety.** No write without an expected revision. Report saved only after persistence succeeds. Every run is bound to a snapshot ID that travels with its logs and waveform.
- **Bridge security.** Bind to loopback only, require an exact origin, pairing code and session token on every message, confine operations to the selected root, reject traversal and symlink escape, and never execute arbitrary commands. Untrusted native code never loads in the browser.
- **Upstream fidelity.** Keep native Xezim defaults and tests intact; browser changes are `cfg(target_arch = "wasm32")`-gated. Pin upstream SHAs and document each patch in `vendor/UPSTREAM.md`. If you change the Surfer `index.html`, update `apps/ide/public/surfer/LOCAL-WRAPPER-SHA256SUMS` with the hash of the committed (LF) file and keep the EUPL-1.2 notice.

### Build and check

```sh
pnpm install --frozen-lockfile
pnpm build:browser                                  # Xezim → apps/ide/public/wasm (ignored output)
pnpm typecheck
pnpm test                                           # protocol, IDE, bridge, wave-index
pnpm build
pnpm test:e2e                                       # Playwright, Chromium + Firefox
cargo test --manifest-path crates/xezim-web/Cargo.toml
cargo test --manifest-path vendor/xezim/Cargo.toml  # when touching vendored Xezim
```

Bridge tests need loopback access. Add positive, negative and recovery tests for each feature. Use native Xezim only as a reference oracle for simulation semantics, with the same sources and options.

### Workflow

- Work on a feature branch; don't push to `main`. Don't edit vendored Xezim core concurrently with another agent; one lane coordinates core changes.
- When a PR or report changes behavior, list the changed files, any new APIs, the exact commands run and their results, any screenshots, known limits, and the upstream SHAs.
- Fail fast on feasibility: show compiler output and a minimal reproducer instead of speculative workarounds.
- Keep `README.md`, this file, `docs/index.html`, `services/mcp-bridge/README.md` and `.agents/skills/` aligned when tools, limits or commands change.
- This repository is pinned as a submodule of the AI Engineer umbrella site. Changes here do not move that pin; the umbrella updates it deliberately.
