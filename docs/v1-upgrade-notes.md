# Xezim Studio v1 upgrade notes

This is the first browser IDE release line. There is no earlier persisted-workspace schema to migrate. Source projects remain ordinary UTF-8 project files stored in the browser's origin-private storage; export a project ZIP before moving to another browser profile or origin.

## Rebuild the browser runtime

Use Node.js 24 or newer in the supported range, pnpm 11.25.0, Rust/Cargo 1.99.1, the `wasm32-unknown-unknown` target, clang, and `wasm-bindgen-cli` 0.2.129. From the repository root:

```sh
pnpm install --frozen-lockfile
cargo install wasm-bindgen-cli --version 0.2.129 --locked
pnpm build:browser
pnpm build
pnpm demo
```

`pnpm build:browser` regenerates ignored files in `apps/ide/public/wasm/` from the pinned vendored sources. `pnpm demo` builds those assets, then serves the IDE on the Vite development URL printed in the terminal. Browser simulation is unavailable if generated assets are missing; no native simulation fallback is used.

## Runtime and data limits

- Browser execution is a one-shot Xezim compile-and-run operation in a Web Worker. The UI and MCP bridge share that operation. Separate compile/run handles, in-run cancellation, `maxSteps`, and a fixed WASM linear-memory ceiling are not available.
- Browser project input is capped at 4,096 files and 16 MiB total UTF-8 path plus contents. The MCP filesystem bridge separately caps snapshots at 4,096 files and 64 MiB total, with individual files capped at 16 MiB; Xezim's lower 16 MiB input cap applies when running them in-browser.
- Stop time is limited to 10,000,000 integer nanoseconds. Fractional nanosecond stop times are rejected.
- Returned stdout is limited by the request to 1 MiB by default and at most 16 MiB. A complete VCD is limited to 64 MiB per artifact; OPFS retains at most 8 artifacts and 128 MiB of VCD data. Browser quota can be lower, and persistent-storage requests can be denied.
- The Worker has a wall-time termination guard, but no hard linear-memory ceiling. Large projects, designs with deep recursion, or large simulations need device-specific validation.
- Supported browser simulation does not include native DPI-C/VPI plugins, Cranelift JIT/AOT, native threads, Python/cocotb, arbitrary host file access, full UVM parity, FST output, or pausable interactive simulation. Do not infer support from native Xezim.

## Native compatibility

The vendored Xezim defaults retain the native allocator and native dependency/build paths. WASM portability gates are target-specific. The `native-xezim` CI job runs the xezim-core, xezim, and wrapper test suites on Linux; check the current CI result before distributing a build.

## Bundled Surfer

Surfer v0.7.0 is pinned to commit `bd749b1f786c1c62cd67893ca71346cbe6983915`. Its EUPL-1.2 license text is distributed at `apps/ide/public/surfer/LICENSE-EUPL-1.2.txt`; upstream and local asset checksums are listed alongside it. The [Surfer upstream project](https://gitlab.com/surfer-project/surfer) identifies Surfer as EUPL-1.2 licensed, and the [European Commission's EUPL-1.2 text](https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12) provides the license terms. Preserve these notices and source provenance when copying the bundle. This repository records the license and notice status; it is not a legal determination that every distribution model or combination of works satisfies the license. Review the intended distribution and source-availability obligations before external release.

## Protocol compatibility

The protocol remains `xezim-ide/1`. `simulation.execute` is the supported browser operation. Existing `simulation.compile` and `simulation.run` schema operations remain present for other backends and future runtime APIs, but this browser implementation reports `UNSUPPORTED_CAPABILITY` for them. MCP `run_simulation` uses the one-shot browser operation and does not fabricate a compile ID.
