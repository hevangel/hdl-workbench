# ADR-002 — Application-facing interface freeze

Date: 2026-10-22

Status: Frozen after review of the completed LUNA-00A and LUNA-00B feasibility results

Related work: [ADR-001 runtime boundary](ADR-001-runtime-architecture.md), [Xezim report](wasm-portability-report.md), [Surfer report](surfer-integration-report.md)

## Context

The application-facing contracts below were reviewed only after both feasibility results were complete:

- Xezim and xezim-core are pinned as a pair. The initial build gate found `getrandom` and `libffi-sys` target blockers; later target-gating and VFS/VCD work produced a WASM release and a real Firefox Worker counter/VCD pass. The full native Xezim suite passed with native filesystem behavior preserved. LUNA-10 added a 256 MiB WebAssembly linear-memory maximum and a 300-second Worker wall-time limit. The combined worker API still does not implement every separate compile/run contract, and no per-event or instruction-count budget is enforced.
- Surfer v0.7.0 is pinned, built, licensed, and tested in a real same-origin browser harness. LUNA-00B loaded a real VCD through HTTP and Blob URLs, added `top.count`, and moved the cursor to 25 ns. LUNA-05 embedded the build, verified zoom and iframe reload readiness, and preserved the license. The WASM SHA-256 is `1341ad01c59f5169209239728417d14f430183abd92515f22bf07cd7c86937ae`. The inner `InjectMessage` API is explicitly unstable; WCP host exports, large-waveform performance, and production Chromium remain unverified. The pinned wrapper has no waveform-load completion acknowledgment, so `loadWaveform` confirms dispatch after readiness, not asynchronous parsing completion.

The application-facing contracts below can be fixed without depending on those implementation details. A frozen interface remains distinct from an implemented capability.

## Decision

Freeze these application-facing contracts for v1. Implementations may report a capability as available only after its corresponding acceptance gate passes. Until then, return an explicit unsupported/unavailable error and keep that feature disabled in the UI and MCP tools. Never route browser simulation to native Xezim as a fallback.

The TypeScript/Zod source of truth and generated Draft 2020-12 wire schemas live in `packages/protocol/`. Its strict object schemas reject unknown fields; semantic validators additionally bind compile/run results to their requested project snapshot and bind RPC responses to the request id and operation. This package implements the application-facing shapes below without claiming either backend is available.

### 1. Workspace

```ts
interface WorkspaceAdapter {
  listFiles(): Promise<FileEntry[]>;
  readFile(path: string): Promise<{ bytes: Uint8Array; revision: string }>;
  writeFile(
    path: string,
    bytes: Uint8Array,
    expectedRevision: string | null,
  ): Promise<{ revision: string }>;
  deleteFile(path: string, expectedRevision: string): Promise<void>;
  snapshot(): Promise<{ snapshotId: string; files: SnapshotFile[] }>;
  subscribeChanges(callback: (change: FileChange) => void): () => void;
}
```

Paths are normalized POSIX-style paths relative to the project. Reject absolute paths, traversal, symlink escape, and policy-disallowed paths. Revisions are opaque. Writes use optimistic concurrency, and the adapter resolves only after durable storage succeeds.

### 2. Browser simulation service

```ts
interface SimulationService {
  compile(input: CompileRequest): Promise<CompileResult>;
  run(input: RunRequest): Promise<RunResult>;
}
```

`CompileRequest` identifies an immutable project snapshot, explicit top module, include directories, defines, and compile mode. `RunRequest` identifies the compiled snapshot, a bounded maximum simulation time represented by integer ticks plus an explicit timescale, a work budget, and optional VCD scope selection. Results carry the source `snapshotId`, diagnostics, captured output, an explicit status, the last simulation tick, and any VCD artifact reference.

The v1 status values are `completed`, `finish`, `failed`, `timeout`, and `resource_limit`. Native-only behavior and unproven browser features return explicit errors; they do not fall through to a native execution path.

`getSignalNow`, `readTransitions`, and `cancel` are outside the frozen v1 interface. Add them only through a reviewed contract change after implementation and dedicated tests. In particular, do not infer historical values from Surfer rendering.

### 3. MCP-to-browser RPC

Control messages use the versioned JSON envelope:

```json
{
  "protocol": "xezim-ide/1",
  "id": "request-uuid",
  "op": "simulation.run",
  "projectId": "active-project-id",
  "params": { "snapshotId": "sha256:...", "until": { "ticks": "1000000", "unit": "ps" } }
}
```

Success responses contain the same `protocol` and `id`, `ok: true`, and a `result`. Error responses contain the same `protocol` and `id`, `ok: false`, and an `{ code, message }` error. Use JSON for control only; waveform bytes travel through the artifact channel, never as large MCP JSON payloads. Require request correlation, bounded timeouts, snapshot provenance, and explicit disconnected/busy/conflict errors.

### 4. Waveform artifact

```ts
type WaveformArtifact = {
  id: string;
  runId: string;
  snapshotId: string;
  format: 'vcd';
  sizeBytes: number;
  storage: 'opfs' | 'bridge-file';
  timescale: {
    magnitude: string;
    unit: 's' | 'ms' | 'us' | 'ns' | 'ps' | 'fs';
  };
  complete: boolean;
};
```

Time values use integer ticks and remain associated with the artifact's declared timescale. Preserve four-state `x` and `z` values. FST and historical query/index fields are not part of v1.

### 5. Surfer adapter

```ts
type WaveformTime = { ticks: string };

interface SurferAdapter {
  ready(): Promise<void>;
  loadWaveform(artifact: WaveformArtifact): Promise<void>;
  addSignals(paths: string[]): Promise<void>;
  setCursor(time: WaveformTime): Promise<void>;
  zoom(start: WaveformTime, end: WaveformTime): Promise<void>;
  dispose(): void;
}
```

`WaveformTime.ticks` is a nonnegative decimal integer in the currently loaded artifact's VCD timescale. Signal paths use the VCD hierarchy. Adapter calls reject when the viewer is not ready, an input is invalid, or the underlying pinned viewer cannot perform the operation.

The adapter is the only module allowed to know the viewer's private messages. This contract does not freeze `LoadUrl`, `InjectMessage`, WCP payloads, a readiness event, Blob URL behavior, or the artifact URL mechanism. Choose and test those against the pinned, self-hosted viewer before enabling the associated adapter capability. Preserve Surfer's EUPL-1.2 license with redistributed assets.

## Conformance gates and capability truthfulness

The v1 application interfaces are frozen; capability availability follows the evidence:

1. **Xezim (bounded worker profile passed):** the pinned parser/core/Xezim checks, WASM release build, Firefox Worker counter/VCD test, and complete native suite passed after the portability patches. See the [build-gate follow-up](wasm-build-gate-followup.md) for the original blockers and [LUNA-04 report](luna-04-virtual-files-report.md) for the build commands; [LUNA-10](luna-10-memory-report.md) records the 256 MiB WebAssembly memory maximum and 300-second Worker wall-time bound. The application still has a combined compile/run entry point rather than complete `SimulationService` mapping; its stdout limit truncates the returned result after collection, so output accumulation is not bounded during simulation, and no per-event or instruction-count budget is enforced. Keep those limits visible and do not claim broader support.
2. **Surfer (load/signal/cursor/zoom passed for one pinned build):** LUNA-00B passed VCD loading, `top.count`, and cursor tick 25 for both same-origin HTTP and Blob URLs. LUNA-05 passed the embedded same-origin fixture, `ZoomToRange` 0–40 ns, and iframe reload/re-handshake in Camoufox Firefox `152.0.4-beta.30`; screenshot evidence and commands are in the [LUNA-05 report](luna-05-surfer-adapter-report.md). Keep the inner `InjectMessage` payloads inside the versioned adapter because upstream marks them unstable. Production Chromium, large-waveform performance, WCP host exports, and load-completion notification remain unverified or unavailable.
3. **Cross-cutting:** keep `readTransitions`, `getSignalNow`, cancellation, FST, and other future capabilities absent until each has an implementation and acceptance tests. Existing interfaces may return explicit `UNSUPPORTED_CAPABILITY` or `SIMULATOR_UNAVAILABLE` errors when a capability is unavailable.

If a pinned backend cannot meet these application-facing contracts, revise this ADR before adding backend-specific behavior to controllers, UI, or MCP tools.
