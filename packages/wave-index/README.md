# HDL Workbench waveform index

This package indexes VCD value changes for historical queries. It does not use pixels from Surfer and does not modify the waveform artifact. Feed it a string for small fixtures or an async iterable of UTF-8 chunks for larger artifacts. `Blob.stream()` can be passed directly in browser runtimes that expose `ReadableStream` async iteration.

```ts
import { parseVcd } from "@xezim-ide/wave-index";

const index = await parseVcd(vcdFile.stream());
const sample = index.querySignal("top.counter", "1500");
const page = index.listTransitions("top.counter", "0", "5000", 250);
const nextPage = page.nextCursor
  ? index.listTransitions("top.counter", "0", "5000", 250, page.nextCursor)
  : undefined;
```

Ticks stay decimal strings at the API boundary and use `BigInt` internally. The index reports the VCD timescale separately; timestamps are not converted to JavaScript floating-point values. `querySignal` returns the last value change at or before the requested tick. When the selected signal has no value yet, `value` is `null`. Queries outside the VCD timestamp range and unknown paths return `WaveformIndexError` codes.

Identifier-code aliases share one change history and are listed on each signal descriptor. VCD `integer` declarations are treated as signed, and `time` declarations as unsigned. Standard VCD does not carry the SystemVerilog `signed` qualifier for `reg` or `wire` vectors; those report `signedness: "unknown"`. Call `querySignal(path, ticks, { signed: true })` when the source declaration supplies that missing metadata. The bit pattern and all X/Z states remain unchanged.

The parser enforces configurable limits for input bytes, signal declarations, vector width, transition count, and estimated index bytes. Transition and signal lists are paged, with opaque cursors bound to one index and query range. Defaults are 256 MiB input, 100,000 signal declarations, 16,384 bits per signal, 1,000,000 changes, and a 128 MiB estimated index budget.

The historical fixture `test/fixtures/four-state.vcd` covers X/Z, an identifier-code alias across scopes, integer signedness, reg signedness override, same-tick transitions, and a 10 ps timescale. `test/wave-index.test.ts` also reads the checked-in Surfer counter fixture. The current workspace has no Cargo/Rust executable, so this package's test suite does not claim to regenerate a native Xezim VCD during its run.
