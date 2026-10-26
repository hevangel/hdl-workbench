---
name: hdl-workbench-cli
description: Drive the HDL Workbench web app from a shell with the hdlwb session CLI, without configuring an MCP server. It writes project files, runs SystemVerilog simulations in the person's paired browser (Xezim WASM), reads logs and diagnostics, and opens waveforms in Surfer. Use it when asked to simulate, test or debug HDL "in the workbench" or "in the browser", or to control HDL Workbench from the command line.
---

# Control HDL Workbench with the `hdlwb` CLI

`hdlwb` (`services/mcp-bridge/scripts/hdlwb.mjs`) keeps **one** MCP bridge running in the background and forwards each command to it. The bridge pairs once with the person's open IDE tab and stays paired across commands. (The MCP Inspector CLI can't do this, because it restarts the bridge on every call and loses the pairing.)

Every simulation runs in the person's browser as Xezim WebAssembly. Neither the CLI nor the bridge simulates anything. Never substitute native `xezim` output for a workbench result.

Run commands from the `hdl-workbench` repository root. Examples use this shell helper:

```sh
hdlwb() { node services/mcp-bridge/scripts/hdlwb.mjs "$@"; }
```

In PowerShell, write `node services/mcp-bridge/scripts/hdlwb.mjs` in place of `hdlwb`.

## 1. Prerequisites

1. Dependencies installed: `pnpm install --frozen-lockfile`.
2. Bridge built: `pnpm --filter @xezim-ide/mcp-bridge build`. If `services/mcp-bridge/dist/main.js` is missing, `hdlwb start` exits with code 2 and prints this command.
3. The IDE is running and open in the person's browser. Check `http://127.0.0.1:5173`. If it isn't running, `pnpm demo` builds the WASM simulator and starts it; that is long-running, so start it in the background or ask the person to. If the IDE is served from another origin, pass it with `--allowed-origins`.

## 2. Start a session and pair

```sh
hdlwb start --project-root /abs/path/to/hdl/project
```

This prints `{ session, projectRoot, log, connection }`, where `connection` is the `get_connection_status` result.

**Pairing needs the person.** It is their consent for the bridge to read and write this project in their browser. Don't automate or bypass it.

1. Give the person the whole `connection` JSON object. Ask them to click **Connect agent** in the IDE, paste it, and press **Pair browser**.
2. The pairing code is single-use and expires after **two minutes**. If it expires, run `hdlwb status` and hand over the fresh JSON.
3. Confirm with `hdlwb status` that the output shows `"connected": true`.

The session lasts until `hdlwb stop`, the bridge exits, or the browser session expires (one hour). Run `hdlwb status` before a batch of work. If `connected` is `false`, pair again.

## 3. Commands

| Command | Does |
| --- | --- |
| `hdlwb status` | Connection state and fresh pairing JSON |
| `hdlwb tools` | Lists the bridge tools |
| `hdlwb cat <path>` | Prints a project file; its `revision` goes to **stderr** |
| `hdlwb put <path> <file\|-> --create` | Creates a new file from a local file or stdin |
| `hdlwb put <path> <file\|-> --expected-revision <rev>` | Replaces a file only if it is still at `<rev>` |
| `hdlwb run --top <module> --until <ticks> [--unit ns] [--magnitude 1]` | Compiles and simulates in the browser. Optional `--wall-ms`, `--max-output-bytes` and `--max-vcd-bytes` set the run's limits. |
| `hdlwb call <tool> '<json>'` | Any tool with raw JSON arguments (or `@file`, or `-` for stdin) |
| `hdlwb stop` | Ends the session and the bridge |

Add `--session <name>` (or set `HDLWB_SESSION`) to run separate sessions for separate projects; the default is `default`.

Output is JSON on stdout, except for `cat`. Exit codes:
- **0**: success.
- **1**: the tool returned an error, as `{ "code", "message" }` on stdout.
- **2**: usage error, no session, or the session couldn't be reached (message on stderr).

Always check the exit code.

Tools reachable through `call`:
- `list_project_files`
- `read_project_file {path}`
- `write_project_file {path, expected_revision, content_base64}`
- `run_simulation {top_module, until_ticks, timescale, work_budget?}`
- `get_simulation_log {run_id, tail_bytes?}`
- `get_diagnostics {run_id?}`
- `list_runs {limit?}`
- `show_waveforms {artifact_id, signal_paths?, cursor?}`
- `get_connection_status {}`

`compile` always returns `UNSUPPORTED_CAPABILITY`; use `run`.

## 4. The working loop

```sh
hdlwb call list_project_files                       # paths + revisions
hdlwb cat rtl/counter.sv 2> rev.txt                 # content; revision saved
# edit locally, then write back against that revision:
hdlwb put rtl/counter.sv ./counter.sv --expected-revision "$(sed 's/^revision //' rev.txt)"
hdlwb run --top tb_counter --until 2000 --unit ns
```

You can also edit files under `--project-root` directly with your normal file tools. The bridge snapshots the disk folder at the start of every run and pushes it to the browser, so disk is the source of truth while paired. The IDE locks ZIP import and file actions during that time. Use `put` with a revision when the person might be editing the same files.

After `run`, read the result:
- `status`: `finish` (`$finish` ran), `completed` (stop time reached), `failed`, `timeout` or `resource_limit`.
- `stdout` / `stderr`: a test passes only if its own output says so, for example a `PASS` line.
- `diagnostics`: file, line and code for each compile or runtime error.
- `waveform.id`: present when the design dumped a VCD.

Show the waveform to the person:

```sh
hdlwb call show_waveforms '{"artifact_id":"<waveform.id>","signal_paths":["tb_counter.dut.count"],"cursor":{"ticks":"150"}}'
```

Signal paths use the VCD hierarchy. Cursor ticks are in the VCD's own timescale.

Report back with the top module, stop time, `status`, the relevant output lines and the `runId`.

## 5. Writing HDL that runs here

- Give testbenches a `` `timescale `` and end with `$finish`. Print an explicit `PASS` or `FAIL: …` line.
- Add `$dumpfile("x.vcd"); $dumpvars(0, <top>);` to get a waveform.
- `$readmemh`, `$fopen` and `$display` resolve **project-relative** paths only.
- Stop time is whole nanoseconds, up to 10,000,000 ns (`--until` × `--magnitude` × `--unit`). Wall time is at most 300 s (`--wall-ms`; the default is 30 s), and simulator memory is 256 MiB.
- DPI: `.c`/`.cpp` files in the project are compiled in the browser. Only `import "DPI-C" function int f(input int …)` works: an `int` return, scalar `input int` arguments, and `extern "C"` in C++. No libc/libc++ calls, DPI tasks, strings, arrays or `output` arguments.
- `run_simulation` accepts `plusargs` (`+name`, `+name=value`, read by `$test$plusargs` and `$value$plusargs`) and `waveform_format` (`vcd` default, or `fst`, converted from the simulator's VCD in the browser). Other command-line flags do not exist in the browser build.
- Not available: native DPI/VPI libraries, cocotb, full UVM, FSDB, pause/resume.

## 6. Errors

| Code / message | Fix |
| --- | --- |
| `BROWSER_DISCONNECTED` | Not paired, or the tab closed. Run `hdlwb status` and pair again (section 2). |
| `REVISION_CONFLICT` | The file changed since you read it. Run `cat` again, merge, and `put` with the new revision. |
| `INVALID_PATH` | Use a project-relative path, without `..` and not absolute. |
| `SIMULATION_BUSY` | A run is in progress. Wait for it, then retry once. |
| `TIMEOUT` | Fix hangs (missing `$finish`, zero-delay loops), shorten `--until`, or raise `--wall-ms` (≤ 300000). |
| `RESOURCE_LIMIT` | Dump fewer scopes, shorten the run, or shrink the design. |
| `SIMULATOR_UNAVAILABLE` | The IDE lacks WASM assets. Ask for `pnpm build:browser`, then reload the page. |
| `DPI_COMPILATION_FAILED` | Fix the C/C++ shown in `diagnostics`. |
| `No running hdlwb session` (exit 2) | Run `hdlwb start --project-root …`. |
| `already running` (exit 2) | Reuse the session, or run `hdlwb stop` first (or use `--session`). |
| `Session failed to start` (exit 2) | Read the printed log tail. Common causes are an unbuilt bridge or a busy `--port` (try `--port 0`). |

Session logs (bridge stderr plus one line per call) are at the `log` path printed by `start`.

## 7. Safety

- The session listens only on `127.0.0.1`. It requires a random token stored in a per-user temp file, and it rejects any request carrying a browser `Origin`. Don't print or copy the token or the state file.
- Don't repeat the pairing code anywhere except to the person, for pasting.
- Run `hdlwb stop` when the work is finished.
