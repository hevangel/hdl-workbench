# Xezim Studio MCP bridge

The bridge exposes the ten Xezim Studio tools over MCP stdio and accepts one paired browser over a WebSocket listener bound to `127.0.0.1`. Filesystem tools operate only within the project root selected with `--project-root` or `XEZIM_PROJECT_ROOT`.

## Build and run

```powershell
pnpm install --frozen-lockfile
pnpm --filter @xezim-ide/mcp-bridge build
node services/mcp-bridge/dist/main.js --project-root B:\path\to\project
```

The default WebSocket URL uses port `47531`; pass `--port 0` to select an available port. Exact browser origins default to `http://localhost:5173` and `http://127.0.0.1:5173`. Configure them with `--allowed-origins` or `XEZIM_ALLOWED_ORIGINS`. The service rejects missing and non-allowlisted `Origin` headers and verifies the session token on every browser message. Pairing codes expire after two minutes and are single-use. The token expires after one hour.

The MCP process writes diagnostics to stderr. Stdout belongs exclusively to the MCP stdio transport.

## MCP Inspector

The Inspector CLI is pinned in this package's dev dependencies. From the repository root:

```powershell
pnpm --dir services/mcp-bridge exec mcp-inspector --cli node dist/main.js --project-root B:\path\to\project --method tools/list
pnpm --dir services/mcp-bridge exec mcp-inspector --cli node dist/main.js --project-root B:\path\to\project --method tools/call --tool-name get_connection_status
```

`get_connection_status` returns the listener URL and current pairing challenge. A browser client must present that challenge from an allowed origin before the browser RPC tools can run.

## Tool inputs

- `get_connection_status`, `list_project_files`: `{}`
- `read_project_file`: `{ "path": "rtl/top.sv" }`
- `write_project_file`: `{ "path": "rtl/top.sv", "expected_revision": "sha256:…", "content_base64": "…" }`. Use `null` for `expected_revision` to create a file; a stale revision returns `REVISION_CONFLICT`.
- `compile`: `{ "top_module": "top", "mode": "full" }` with optional `include_dirs` and `defines`.
- `run_simulation`: `{ "compile_id": "…", "until_ticks": "1000", "timescale": { "magnitude": "1", "unit": "ns" } }` with optional resource budgets and VCD scopes.
- `get_diagnostics`: optional `compile_id` and/or `run_id`.
- `list_runs`: optional `limit`.
- `show_waveforms`: `{ "artifact_id": "…" }` with optional signal paths and cursor ticks.
- `get_simulation_log`: `{ "run_id": "…" }` with optional `tail_bytes`.

Compile and run require a paired browser. Before compile, the bridge synchronizes the selected project snapshot through the frozen `workspace.list_files`, `workspace.write_file`, and `workspace.delete_file` RPCs. It does not invoke a native simulator.

## Tests

```powershell
pnpm --filter @xezim-ide/mcp-bridge test
pnpm --filter @xezim-ide/mcp-bridge test:inspector
```

The tests exercise stdio MCP tool discovery/calls, the pinned Inspector CLI, WebSocket pairing, origin and token rejection, pairing expiry, project path traversal, symlink escapes, revision conflicts, snapshot synchronization, browser disconnects, request timeouts, and deterministic simulation busy errors.
