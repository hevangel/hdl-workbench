# ADR-002 — Application-facing interface freeze

Date: 2026-10-21

Status: Accepted after review of LUNA-00A and LUNA-00B

Related work: [ADR-001 runtime boundary](ADR-001-runtime-architecture.md), [Xezim report](wasm-portability-report.md), [Surfer report](surfer-integration-report.md)

## Context

The two feasibility spikes found enough to settle the application boundaries, but neither backend passed its runtime gate:

- Xezim and xezim-core are pinned, and source inspection found plausible single-thread and inline-VCD paths. The Rust toolchain was unavailable, so parser, core, and Xezim WASM checks did not reach compilation.
- Surfer source is pinned and its wrapper documents `LoadUrl`. The artifact itself is not pinned, and no browser run verified VCD loading, signal addition, or cursor control. `InjectMessage` forwards an upstream internal API that is explicitly unstable; the proposed add-signal and cursor payloads remain unverified.

The application-facing contracts below can be fixed without depending on those implementation details. A frozen interface is not evidence that a backend capability is available.

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

The v1 application interfaces are frozen; backend conformance is not:

1. **Xezim (partial):** `sv-parser` passed on Rust 1.99.1. Baseline core and reduced Xezim checks fail in `getrandom` 0.2.17/0.3.4. A disposable browser-feature setup lets core compile, then Xezim fails in `libffi-sys` because Autoconf rejects `wasm32-unknown-unknown`. See the [build-gate follow-up](wasm-build-gate-followup.md). Target-gate native FFI and build-script code, preserve native defaults, then build the browser Worker counter fixture. Verify VCD output, output/work limits, and native regression behavior before exposing browser compile/run as available.
2. **Surfer:** build from the pinned source or obtain an immutable asset, record its digest and license, then run a real-browser test that opens the VCD fixture, adds `top.count`, and positions the cursor at tick 25. Do not enable the corresponding adapter methods until the viewer passes.
3. **Cross-cutting:** keep `readTransitions`, `getSignalNow`, cancellation, FST, and other future capabilities absent until each has an implementation and acceptance tests. Existing interfaces may return explicit `UNSUPPORTED_CAPABILITY` or `SIMULATOR_UNAVAILABLE` errors while a gate is open.

If a pinned backend cannot meet these application-facing contracts, revise this ADR before adding backend-specific behavior to controllers, UI, or MCP tools.
