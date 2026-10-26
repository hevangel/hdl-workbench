# HDL Workbench

HDL Workbench is a browser-based SystemVerilog workspace. You edit HDL in the browser, the [Xezim](https://github.com/aionhw/xezim) simulator compiles and runs it as WebAssembly inside a browser Web Worker, and an embedded [Surfer](https://gitlab.com/surfer-project/surfer) viewer shows the resulting waveform. Projects persist in the browser between sessions.

An AI coding agent (for example Codex or Claude Code) can optionally drive the same browser session through a local MCP bridge: it reads and writes project files on disk, runs the simulation in your browser, reads logs and diagnostics, and opens waveforms in Surfer.

> **Simulation always runs in the browser.** There is no server-side or native fallback. If the WASM simulator assets are missing, the Run button is disabled rather than quietly using something else.

For a visual walkthrough of how the pieces fit together, open [`docs/index.html`](docs/index.html) in a browser.

## What you can do

- Edit SystemVerilog in a Monaco editor with syntax highlighting and inline diagnostics.
- Run a bounded, one-shot compile-and-simulate of a multi-file project, including `$readmemh`, `$fopen`, `$display` and VCD dumping against project-relative files.
- View the VCD in Surfer: add signals, move the cursor, zoom.
- Call small C or C++ helpers through SystemVerilog DPI. They are compiled to WebAssembly in the browser (see [limits](#limits)).
- Keep projects in the browser's private storage, and import or export them as ZIP files.
- Pair a local AI agent over MCP so it can edit, run and inspect the project with you watching.

## Quick start

Prerequisites: Node.js 24–26, pnpm 11.25.0, Rust 1.99.1 with the `wasm32-unknown-unknown` target (pinned by `rust-toolchain.toml`), clang, and `wasm-bindgen-cli` 0.2.129.

```sh
pnpm install --frozen-lockfile
cargo install wasm-bindgen-cli --version 0.2.129 --locked
pnpm demo
```

`pnpm demo` builds the Xezim WASM simulator into `apps/ide/public/wasm/` and starts the IDE on the local Vite URL it prints (normally `http://127.0.0.1:5173`). Open that URL, write or open a module, set **Top module**, and press **Run**. Output appears in the bottom panel and the waveform opens in Surfer.

Use a current Chromium-based browser or Firefox on `localhost` or HTTPS; the workspace needs the browser's Origin Private File System.

## Working with an AI agent

1. Build the bridge: `pnpm --filter @xezim-ide/mcp-bridge build`.
2. Register it as an MCP stdio server in your agent, pointing at the folder the agent may edit:

   ```sh
   node services/mcp-bridge/dist/main.js --project-root /path/to/your/hdl/project
   ```

3. Ask the agent to call `get_connection_status`. It returns a JSON object with a WebSocket URL and a short-lived pairing code.
4. In the IDE, click **Connect agent**, paste that JSON, and click **Pair browser**.

Agents that only have a shell can use the `hdlwb` session CLI instead of an MCP configuration. `node services/mcp-bridge/scripts/hdlwb.mjs start --project-root <dir>` keeps one paired bridge running in the background, and later `hdlwb status`, `hdlwb run …` and `hdlwb call …` commands reuse it. The [hdl-workbench-cli skill](.agents/skills/hdl-workbench-cli/SKILL.md) documents it.

The agent can now list, read and write project files, run the simulation in your browser tab, read logs and diagnostics, and show waveforms. The bridge listens only on `127.0.0.1`, accepts only allow-listed browser origins, is confined to the project root you gave it, and never runs a simulator itself. Pairing codes expire after two minutes; sessions after one hour. See [`AGENTS.md`](AGENTS.md) for the agent-side workflow and [`services/mcp-bridge/README.md`](services/mcp-bridge/README.md) for every option.

## Limits

This is a bounded first release; these are deliberate, tested boundaries:

| Area | Limit |
| --- | --- |
| Run model | One-shot compile and run. No pause, resume or in-run cancellation. |
| Stop time | Integer nanoseconds, at most 10,000,000 ns. |
| Wall time | 300 seconds per run, enforced by terminating the Worker. |
| Simulator memory | 256 MiB WebAssembly linear memory. Exceeding it reports `RESOURCE_LIMIT`. |
| Project input | 4,096 files and 16 MiB total in the browser simulator. |
| Waveforms | VCD only, up to 64 MiB each; 8 runs and 128 MiB retained. |
| DPI | Functions only, returning `int` with scalar `input int` arguments. C++ functions need `extern "C"`. No `.so`/`.dll` loading. |
| Not supported | Native DPI/VPI plugins, Python/cocotb, full UVM, FSDB, command-line flags beyond `+plusargs` and `-sv_seed`, multithreaded or JIT simulation. |

## Repository layout

| Path | Contents |
| --- | --- |
| `apps/ide/` | The browser IDE (React, Vite, Monaco), simulator and compiler Workers, Surfer embedding |
| `crates/xezim-web/` | Rust wrapper that exposes Xezim to the browser through wasm-bindgen |
| `packages/protocol/` | Versioned `xezim-ide/1` wire schemas shared by the IDE and the bridge |
| `packages/wave-index/` | VCD indexer for historical signal queries |
| `services/mcp-bridge/` | Local MCP stdio server and loopback WebSocket bridge to the browser |
| `vendor/` | Pinned Xezim and xezim-core sources with documented browser patches |
| `spikes/` | Early feasibility experiments, kept for reference |
| `docs/` | The visual overview (`index.html`) and the [initial build history](docs/session_history/initial_build/) |

## Development and tests

```sh
pnpm typecheck
pnpm test          # unit and integration tests for every workspace package
pnpm build         # production build and generated JSON schemas
pnpm test:e2e      # Playwright: MCP edit → browser Xezim → Surfer, in Chromium and Firefox
cargo test --manifest-path crates/xezim-web/Cargo.toml
```

Some bridge tests bind to loopback ports, and the end-to-end suite needs the Playwright browsers installed. CI runs in [`.github/workflows/luna09-browser-e2e.yml`](.github/workflows/luna09-browser-e2e.yml).

## History and design records

The design decisions (ADRs), the original implementation plan, and every milestone report from the initial build are in [`docs/session_history/initial_build/`](docs/session_history/initial_build/). Start with [`v1-release-readiness.md`](docs/session_history/initial_build/v1-release-readiness.md) for the pinned versions and acceptance results, and [`v1-upgrade-notes.md`](docs/session_history/initial_build/v1-upgrade-notes.md) for runtime limits.

## Licensing

HDL Workbench is MIT licensed (see [`LICENSE`](LICENSE)). Xezim and xezim-core are `MIT OR Apache-2.0` ([`vendor/UPSTREAM.md`](vendor/UPSTREAM.md)). The bundled Surfer v0.7.0 is EUPL-1.2, and its notice and modified entry point are recorded in [`apps/ide/public/surfer/`](apps/ide/public/surfer/UPSTREAM.md). Third-party compiler notices are in [`apps/ide/public/licenses/THIRD_PARTY.md`](apps/ide/public/licenses/THIRD_PARTY.md). A qualified license review of the combined application is still pending before any public hosted release.
