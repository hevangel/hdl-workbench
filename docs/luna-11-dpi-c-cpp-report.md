# LUNA-11 — Browser DPI and C/C++ compilation

Status: implemented and locally verified on 2026-10-22. This is a limited
browser ABI profile; it does not add native simulator behavior to the browser.

## What works

- Xezim WASM accepts a DPI-C function only when it returns SystemVerilog `int`
  and every parameter is scalar `input int`.
- Project `.c`, `.cc`, `.cpp`, and `.cxx` source files compile in a dedicated
  browser Worker with pinned `browsercc` 0.1.1 (Clang/LLD). C is compiled as C,
  and C++ is compiled as C++20. Header files are passed as virtual include files.
- Each translation unit becomes its own standalone Wasm module. Modules may
  not import host functions, and a DPI symbol must resolve to one unique export.
- C++ functions imported by DPI must use `extern "C"` so their exported names
  match the C symbol name Xezim resolves.
- The simulator worker parses module signatures before invoking the export;
  each called DPI export must have a WebAssembly `(i32, ...) -> i32` signature.
  Module memory must be unshared, 32-bit, bounded, and no larger than 64 MiB.
- The worker converts compile, ABI, trap, resource, and missing-symbol failures
  to explicit errors. No native simulator fallback is used.

The browser acceptance fixture proves a C `add_ints` call returns 30 and a C++
`add_ints_cpp` call returns 42 from SystemVerilog. It also verifies a C syntax
error and a successful compile/simulation after that error.

## Current limits

- DPI tasks, output/ref/inout arguments, strings, real values, vectors,
  aggregates, open arrays, callbacks into SV, and time-suspending calls are
  unsupported.
- The current C++ flags disable exceptions and RTTI. The link uses `-nostdlib`
  and rejects all module imports, so code that needs libc/libc++, WASI calls, or
  unresolved cross-translation-unit symbols will fail. Header-only code that
  compiles and links without a runtime may work; this is not a general-purpose
  hosted C/C++ application runtime.
- Source/header inputs are limited to 64 files, 16 translation units, 4 MiB per
  file, and 16 MiB total. Each generated module is limited to 16 MiB. The
  project may load up to 32 modules / 32 MiB total.
- Exports used as DPI calls must return exactly one `i32` and take only `i32`
  parameters. Xezim’s SystemVerilog side supports only signed scalar `int`.
- The first-use compiler assets total 94,377,252 bytes (94.4 MB / 90.0 MiB)
  before HTTP compression. The unused 19.4 MB C++ PCH is excluded from the app
  bundle. A compiler-inclusive process profile on the selected workstation is
  recorded in [`luna-12-dpi-memory-report.md`](luna-12-dpi-memory-report.md);
  it does not qualify lower-resource devices or Firefox process memory.

## Pins and notices

- `browsercc` npm package: 0.1.1, integrity
  `sha512-zKSm2b10XjqRBPkY67rS5FK+YgcXd5CH2GRVwyK2+f7EOQRmHR0IN2lQCvJZQfTytSFbwFmlIpZaxSMFcE+jxw==`.
- Its build script pins LLVM `llvmorg-20.1.2` and downloads WASI SDK sysroot
  25.0. The package MIT license is copied to
  `apps/ide/public/licenses/browsercc-MIT.txt`.
- The identified LLVM, WASI SDK, wasi-libc, and Emscripten license texts and
  generated asset hashes are served from `/licenses/`; see
  [`apps/ide/public/licenses/THIRD_PARTY.md`](../apps/ide/public/licenses/THIRD_PARTY.md).

The packed `browsercc` npm tarball includes LLVM and a WASI sysroot but does not
include a component manifest. The sysroot archive contains no notice files, and
the upstream build container uses the mutable Emscripten `latest` channel.
Therefore the notice work is a technical inventory, not a complete audit or a
qualified legal review. The app can be built and tested locally; this report
does not claim legal approval or a deployment.

## Reproduce

Install the pinned npm package and run the app checks:

```powershell
pnpm install --frozen-lockfile
pnpm --filter @xezim-ide/app typecheck
pnpm --filter @xezim-ide/app test
pnpm --filter @xezim-ide/protocol test
pnpm --filter @xezim-ide/app build
```

Rebuild the patched Xezim web module in the project builder:

```sh
docker exec xezim-luna04-builder sh -lc 'export PATH=/usr/local/cargo/bin:$PATH && cd /host && cargo build --manifest-path crates/xezim-web/Cargo.toml --target wasm32-unknown-unknown && wasm-bindgen --target web --out-dir apps/ide/public/wasm crates/xezim-web/target/wasm32-unknown-unknown/debug/xezim_web.wasm'
```

For the real-browser acceptance, start Chrome with remote debugging on port
9223, then run `pnpm --filter @xezim-ide/app exec vite preview --host 127.0.0.1
--port 4180 --strictPort`. In another shell:

```powershell
$env:XEZIM_CHROME_DEVTOOLS = 'http://127.0.0.1:9223'
$env:XEZIM_BROWSER_TEST_URL = 'http://127.0.0.1:4180/worker-harness.html'
node apps/ide/scripts/wasm-browser-acceptance.mjs
```

Verified on Windows with Headless Chrome 156.0.8078.47. The harness passed the
existing counter, project/VCD, negative compile/recovery, C compile, C++ compile,
direct export, and SystemVerilog DPI callback checks.

The same real worker harness now runs under both configured Playwright browser
projects. The cross-browser test and full-suite results are recorded in
[`luna-13-dpi-cross-browser-report.md`](luna-13-dpi-cross-browser-report.md).

## Native compatibility check

The native `xezim` DPI binding code remains behind the existing non-WASM cfg.
Run the focused native integration test in the builder to verify the original
shared-library path:

```sh
docker exec xezim-luna04-builder sh -lc 'export PATH=/usr/local/cargo/bin:$PATH && cd /host && cargo test --manifest-path vendor/xezim/Cargo.toml --features libffi/system --test strings dpi_simple_test -- --nocapture'
```

The builder's default vendored libffi configure path failed before tests started
because its generated build directory lacked `config.log`. The system-libffi
rerun used libffi 3.4.8 already installed in the builder and passed
`dpi_integration_tests::dpi_simple_test` (1 passed, 251 filtered out).
