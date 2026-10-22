# LUNA-10 browser memory validation

Date: 2026-10-22
Branch: `codex/luna-10-hardening`

## Result

The browser Xezim artifact is linked with a 256 MiB maximum WebAssembly linear memory. The Web Worker converts allocation traps near that limit into a `RESOURCE_LIMIT` response and is terminated after each request, so a subsequent small simulation starts in a fresh worker.

The limit is scoped to WebAssembly linear memory. It does not bound the browser process, JavaScript strings, VCD buffers, browser storage, or other process memory. A minimum supported device profile has not yet been defined, so these desktop measurements do not close the minimum-device release gate.

## Validation host reference

The host used for these browser checks reported Windows `10.0.26300`, 63.9 GiB RAM, an AMD Ryzen 9 5950X, and 32 logical processors. Reproduce the inventory with:

```powershell
node -e 'const os=require("node:os"); console.log(JSON.stringify({platform:os.platform(),release:os.release(),totalRamGiB:+(os.totalmem()/1024**3).toFixed(1),logicalCpuCount:os.cpus().length,cpuModel:os.cpus()[0]?.model},null,2))'
```

This is a high-resource reference workstation, not the v1 minimum. The whole-browser profiles below are sampled peaks, not guaranteed absolute high-water marks. No device support claim should be inferred from these figures.

## Whole-browser process profile (reference host)

The profiler runs the same 4.3-million-word direct WASM allocation trap, 1-million-word Worker completion, over-limit Worker `RESOURCE_LIMIT`, and fresh-Worker recovery scenario while sampling the launched browser's process set. Chromium process IDs come from its browser CDP session; Firefox is scoped to the Playwright-launched process tree. On Windows, resident and private bytes are collected every 50 ms from those processes. The aggregate resident-set sum can count shared pages more than once; private bytes are a second measure where available.

Reproduce with installed Playwright browsers and local-loopback access:

```powershell
pnpm --filter @xezim-ide/mcp-bridge profile:browser-memory -- chromium
pnpm --filter @xezim-ide/mcp-bridge profile:browser-memory -- firefox
```

Results from this host:

| Browser | Browser processes | Samples | Peak aggregate resident | Peak aggregate private |
| --- | ---: | ---: | ---: | ---: |
| Chromium | 4 | 12 | 844.1 MiB | 796.3 MiB |
| Firefox | 7 | 8 | 1,247.0 MiB | 1,317.6 MiB |

Both profiles observed the 255,524,864-byte direct WASM allocation trap, successful supported Worker workload, over-limit `RESOURCE_LIMIT`, and recovery run. These desktop figures do not close the minimum-device gate; repeat on the product's chosen minimum browser/device profile before making memory claims.

## Reproduce the browser WASM build

Use Rust/Cargo 1.99.1 and wasm-bindgen CLI 0.2.129:

```sh
cargo build --locked --release \
  --manifest-path crates/xezim-web/Cargo.toml \
  --target wasm32-unknown-unknown
mkdir -p apps/ide/public/wasm
wasm-bindgen --target web --out-dir apps/ide/public/wasm \
  crates/xezim-web/target/wasm32-unknown-unknown/release/xezim_web.wasm
```

The target-specific linker maximum is in `.cargo/config.toml`; it does not apply to native targets. In this Windows environment, the pinned build and native tests ran in the Debian builder using a Linux scratch copy. Recreate the current source snapshot and run the native acceptance commands with:

```sh
docker exec xezim-luna04-builder sh -lc 'mkdir -p /tmp/xezim-studio-luna10 && tar --exclude=target --exclude=.git -cf - -C /host crates/xezim-web vendor/xezim vendor/xezim-core .cargo rust-toolchain.toml | tar -xf - -C /tmp/xezim-studio-luna10 && cd /tmp/xezim-studio-luna10 && PATH=/usr/local/cargo/bin:$PATH /usr/local/cargo/bin/cargo test --manifest-path vendor/xezim-core/Cargo.toml -- --test-threads=1 && PATH=/usr/local/cargo/bin:$PATH /usr/local/cargo/bin/cargo test --manifest-path vendor/xezim/Cargo.toml -- --test-threads=1 && PATH=/usr/local/cargo/bin:$PATH /usr/local/cargo/bin/cargo test --manifest-path crates/xezim-web/Cargo.toml --lib -- --test-threads=1'
```

This runs Cargo against `/tmp` in Linux instead of the Windows-hosted `/host` filesystem, which avoids the `libffi-sys` Autoconf failure documented in the [LUNA-04 report](luna-04-virtual-files-report.md).

## Reproduce the cap and recovery acceptance test

Install workspace dependencies and Playwright browsers as described by the repository setup, then run with local loopback access enabled for Playwright's Vite server:

```sh
pnpm --filter @xezim-ide/mcp-bridge exec playwright test \
  e2e/luna10-memory.spec.mjs --project=chromium
pnpm --filter @xezim-ide/mcp-bridge exec playwright test \
  e2e/luna10-memory.spec.mjs --project=firefox
```

The test loads the built WASM artifact in a real browser, measures a SystemVerilog RAM fixture, submits the same over-limit design through the actual Xezim Worker client, checks for `RESOURCE_LIMIT`, then verifies that a fresh Worker can run a small design.

Both commands passed on Chromium `156.0.8078.4` and Firefox `157.0`.

## Real Codex CLI and browser smoke

With a logged-in Codex CLI, built WASM assets, installed Playwright browsers, and local-loopback access, run:

```powershell
pnpm --filter @xezim-ide/mcp-bridge smoke:codex
```

The command uses `codex exec --ephemeral` with a read-only shell and per-invocation MCP configuration. It launches the IDE in Chromium, pairs the browser from the actual `get_connection_status` response, asks Codex to update a temporary project file, and verifies the resulting Xezim simulation output. It does not persist Codex settings or source files. This passed with Codex CLI `0.163.0` and Chromium `156.0.8078.4`.

The bridge's pinned MCP Inspector test also passed independently:

```powershell
pnpm --filter @xezim-ide/mcp-bridge test:inspector
```

Its package script uses a no-space test-name pattern so PowerShell and POSIX shells select the test consistently.

## Measurements

The measured high-water values are `WebAssembly.Memory.buffer.byteLength` after simulation. A one-shot source declares a 32-bit RAM, writes one word, and finishes. The browser memory allocator grows in steps, so these are workload-specific figures, not a conversion factor for arbitrary designs.

| Declared RAM | Raw array size | Browser result |
| --- | ---: | --- |
| 1,048,576 × 32 bits | 4 MiB | 68,419,584 bytes (65.3 MiB), completed in Chromium and Firefox |
| 4,194,304 × 32 bits | 16 MiB | 266,600,448 bytes (254.3 MiB), completed in Chromium |
| 4,300,000 × 32 bits | 16.4 MiB | Allocation trap at 255,524,864 bytes (243.7 MiB); `RESOURCE_LIMIT`; fresh-worker recovery passed in Chromium and Firefox |

The 4.3-million-word attempt traps before the linear memory reaches its declared maximum because the next allocator growth request cannot fit. The Worker therefore identifies a WebAssembly runtime trap only when memory has already grown to within 32 MiB of the 256 MiB ceiling. Other traps remain `INTERNAL_ERROR`.

## Native compatibility

The 256 MiB maximum is passed only for `wasm32-unknown-unknown` by target-specific Cargo rustflags. The Linux native test commands are unchanged:

```sh
cargo test --manifest-path vendor/xezim-core/Cargo.toml -- --test-threads=1
cargo test --manifest-path vendor/xezim/Cargo.toml -- --test-threads=1
cargo test --manifest-path crates/xezim-web/Cargo.toml --lib -- --test-threads=1
```

Results from the current branch:

| Native command | Result |
| --- | --- |
| `cargo test --manifest-path vendor/xezim-core/Cargo.toml -- --test-threads=1` | Passed; 71 core unit tests plus the core integration targets. |
| `cargo test --manifest-path vendor/xezim/Cargo.toml -- --test-threads=1` | Passed; complete Xezim native suite, including the 705-test integration target, 4 UPF tests, and doctests. |
| `cargo test --manifest-path crates/xezim-web/Cargo.toml --lib -- --test-threads=1` | Passed; 7 wrapper tests. |
