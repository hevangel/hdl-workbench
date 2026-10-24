# LUNA-08 historical waveform query report

## Result

Implemented a streaming TypeScript VCD index in `packages/wave-index/`. It parses scope hierarchy, declaration identifier codes, aliases, VCD timescale, timestamps, scalar values, and four-state vectors. It preserves BigInt tick precision and pages both signal metadata and per-signal transitions. Query errors identify invalid times, out-of-range requests, unknown signals, malformed files, invalid cursors, and resource limits.

`querySignal(path, ticks)` returns the last value change at or before the requested tick. `listTransitions(path, start, end, pageSize, cursor?)` returns ordered VCD change records, including multiple changes at one tick. Its cursor is bound to the selected signal, tick range, and index instance. The source is consumed as bounded UTF-8 chunks; the parsed index does not retain the original VCD text.

## Reproducible commands

```powershell
pnpm install --frozen-lockfile
pnpm --filter @xezim-ide/wave-index test
pnpm --filter @xezim-ide/wave-index typecheck
pnpm --filter @xezim-ide/wave-index build
```

## Test coverage

- Four-state queries at exact and intermediate ticks; X and Z survive unchanged.
- Hierarchical scopes and three aliases for one VCD identifier code resolve to the same event history.
- Multiple changes at the same tick are retained in source order; exact-tick lookup returns the last one.
- `$timescale 10 ps` is preserved, and time ticks remain decimal strings.
- VCD `integer` values are interpreted as signed; `reg` vectors expose unknown signedness unless the caller supplies source metadata.
- Transition cursors page without skipping repeated-tick changes; signal metadata pages a 1,205-signal input across calls.
- UTF-8 byte chunks split tokens at arbitrary boundaries.
- Unknown signals, invalid time ranges, malformed declarations, excessive input, and excessive change counts return typed errors.
- The checked-in Surfer counter VCD fixture returns the expected count at tick 0 and tick 5.

## Native-fixture validation gap

The requested comparison against a VCD emitted by the native Xezim simulator remains unverified in this environment. `crates/xezim-web/src/lib.rs` contains native simulation tests and expected X/Z/count transitions, but this checkout has no `cargo`, `rustc`, or `rustup` executable on `PATH` or under the inspected user toolchain directories. The Surfer fixture is an existing sample VCD, and `test/fixtures/four-state.vcd` is a deterministic conformance fixture; neither is represented as a fresh native Xezim output. Run the native fixture generator/compare once the Rust toolchain is available, and preserve the generated artifact plus source revision in this package.

## Signedness limit

Standard VCD does not preserve signedness for `reg` and `wire` declarations. The index interprets VCD `integer` types as signed and supports an explicit signedness override for vectors when the caller has source declaration metadata. Without that metadata, it reports `unknown` instead of guessing.
