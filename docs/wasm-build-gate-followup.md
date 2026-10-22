# Xezim WASM build gate follow-up

Date: 2026-10-22

Branch: `codex/luna-00a-wasm-build-gate`

## Result

The earlier environment blocker was resolved by running the checks in the official Rust Linux container. The pinned parser builds for `wasm32-unknown-unknown`. The pinned `xezim-core` and reduced Xezim crate checks both fail during dependency compilation because the graph includes `getrandom` versions that do not support this target without browser-specific configuration.

This is the first compiler evidence against the pinned dependency graph. It does **not** establish that Xezim itself cannot be ported. The failure happens before compiling the Xezim or core crate sources, so later blockers such as core's required `zstd`, Xezim's C shims, native linker arguments, and unconditional FFI dependencies remain untested by these commands.

## Reproduction

Source checkouts:

- Xezim: `22faa127a9cc0156bd23e0ab117f400c64231913`
- xezim-core (including `xezim-parser`): `d15cdc39e27a4c0a8a55f7e1d46369181307ead9`

The checks used Rust `1.99.1` in the official image digest `rust@sha256:a5ab4292e80567dc92f642e2a7d63773ebd695b82b56e8f06dc8d6ecc5600b76`. Set `$srcRoot` to the directory containing the two pinned checkouts.

```powershell
$srcRoot = 'B:\xezim-studio\.codex-rust-luna00\src'
$image = 'rust@sha256:a5ab4292e80567dc92f642e2a7d63773ebd695b82b56e8f06dc8d6ecc5600b76'
docker run --rm -v "${srcRoot}:/src" $image sh -c 'rustup target add wasm32-unknown-unknown && cargo check --manifest-path /src/xezim-core/xezim-parser/Cargo.toml --target wasm32-unknown-unknown'
docker run --rm -v "${srcRoot}:/src" $image sh -c 'rustup target add wasm32-unknown-unknown && cargo check --manifest-path /src/xezim-core/Cargo.toml --target wasm32-unknown-unknown --no-default-features'
docker run --rm -v "${srcRoot}:/src" $image sh -c 'rustup target add wasm32-unknown-unknown && cargo check --manifest-path /src/xezim/Cargo.toml --target wasm32-unknown-unknown --no-default-features --lib'
```

Observed results:

| Check | Result |
|---|---|
| `sv-parser` | Passed for `wasm32-unknown-unknown` on Rust 1.99.1. |
| `xezim-core --no-default-features` | Failed in `getrandom 0.2.17`: `wasm*-unknown-unknown` is unsupported without the `js` feature. |
| `xezim --no-default-features --lib` | Failed in both `getrandom 0.2.17` and `getrandom 0.3.4`; the latter requests the `wasm_js` backend configuration. |

Cargo feature-tree commands confirm the dependency paths:

```sh
cargo tree --manifest-path /src/xezim-core/Cargo.toml --target wasm32-unknown-unknown -e features -i getrandom@0.2.17
cargo tree --manifest-path /src/xezim-core/Cargo.toml --target wasm32-unknown-unknown -e features -i getrandom@0.3.4
```

- `getrandom 0.2.17` is enabled through `rand 0.8` → `rand_core 0.6`.
- `getrandom 0.3.4` is enabled through `ahash 0.8` default `runtime-rng` features.
- Xezim pulls the same paths from its own `rand`/`ahash` dependencies and from xezim-core.

The `--no-default-features` check only removes xezim-core's optional mimalloc allocator; it does not disable these random-number dependencies. Do not enable browser entropy or remove random-seed support without checking simulator semantics and reproducibility requirements.

## Windows host note

Rust was also installed in a task-scoped local directory, but the host had no MSVC build tools. Its `link.exe` resolved to `C:\Program Files\coreutils\bin\link.exe`, which is not the MSVC linker, so host build scripts could not link. The Linux container provided a working Rust host and the proper WASM standard library; the parser check then passed.

## Next patch experiment

In a separate Xezim portability branch, prototype target-specific browser features for the two `getrandom` paths or replace entropy-dependent defaults with injected/deterministic seeds where behavior permits. Preserve native defaults. Re-run the three checks and inspect subsequent errors before changing build scripts or runtime file/thread behavior. Keep this follow-up report as the baseline; the existing pinned upstream checkouts were not modified.
