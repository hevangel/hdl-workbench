# LUNA-07 MCP + WebSocket bridge report

## Result

Implemented on branch `codex/luna-07-mcp-bridge`. The bridge exposes all ten requested MCP tools through the official MCP TypeScript SDK stdio transport and uses a local `ws` listener bound specifically to IPv4 `127.0.0.1`. The protocol envelopes and file operations reuse `@xezim-ide/protocol` schemas and RPC operations.

The MCP process requires an explicitly selected project root (`--project-root` or `XEZIM_PROJECT_ROOT`). Project paths are project-relative POSIX paths. The server checks lexical traversal and resolved symlink targets, caps file reads/writes at the shared 16 MiB protocol limit, and uses SHA-256 revisions with optimistic conflict checks. Writes use a same-directory temporary file and recheck the target revision before replacement.

Browser connections require an exact allowlisted `Origin`, a short-lived single-use pairing code, and a random session token. The token is checked on every message. Pairing expires after two minutes; sessions expire after one hour. A single browser session is accepted. Requests use UUID IDs, a configurable timeout, explicit disconnect errors, and a deterministic busy result for overlapping compile/run work.

Compile and run are forwarded only to the paired browser. Before compile, files are synchronized through the already-frozen list/write/delete workspace RPCs and tied to an immutable snapshot ID. No native simulator fallback exists. The integration tests use a mock browser; this package does not claim real Xezim browser simulation or IDE pairing UI integration.

## Reproducible commands

```powershell
pnpm install --frozen-lockfile
pnpm --filter @xezim-ide/mcp-bridge test
pnpm --filter @xezim-ide/mcp-bridge test:inspector
pnpm --filter @xezim-ide/mcp-bridge typecheck
pnpm --filter @xezim-ide/mcp-bridge build
```

The full acceptance suite launches the built stdio service and pairs a mock browser over `127.0.0.1`. The Inspector acceptance test invokes `get_connection_status` through the pinned MCP Inspector CLI. In this sandbox, the tests require local-network execution permission because the sandbox denies loopback connections; they passed with the command runner's elevated local-network access.

## Findings and test coverage

- Verified ten MCP tools are discoverable through the official SDK client and Inspector CLI.
- Verified `get_connection_status`, file listing/read/write, browser snapshot synchronization, compile, run, diagnostics, run listing, waveform display, and simulation logs.
- Verified stale file revisions return `REVISION_CONFLICT`.
- Verified an untrusted origin receives HTTP 403; pre-pair messages and invalid session tokens close with WebSocket code 4401.
- Verified traversal and absolute paths are rejected by tool input validation; a junction to an external directory returns `INVALID_PATH`.
- Verified expired pairing codes are rejected.
- Verified disconnect returns `BROWSER_DISCONNECTED`, ignored browser requests return `TIMEOUT`, and overlapping compile/run work returns `SIMULATION_BUSY`.
- Confirmed server startup and bridge logs go to stderr while MCP stdio remains usable by both the SDK client and Inspector.
- Dependencies are exact-pinned: MCP server/client SDK `2.3.1`, MCP Inspector `2.12.0`, `ws` `8.22.0`, Zod `4.6.7`.

## Limits and next work

- The IDE does not yet expose the pairing UI or WebSocket client. LUNA-09 must connect the browser app to the returned pairing challenge and bridge URL.
- Compile/run behavior is validated against a mock browser only. LUNA-09 must route the real Xezim WASM worker and artifact store through the same RPCs.
- The bridge supports one selected project root and one browser at a time. Browser origins must be configured for any non-default Vite/production origin.
- The LUNA-07 package validates VCD handoff but does not implement historical waveform queries; that belongs to LUNA-08.

## Inspector invocation

```powershell
pnpm --dir services/mcp-bridge exec mcp-inspector --cli node dist/main.js --project-root B:\path\to\project --method tools/list
pnpm --dir services/mcp-bridge exec mcp-inspector --cli node dist/main.js --project-root B:\path\to\project --method tools/call --tool-name get_connection_status
```
