# Xezim WASM build gate follow-up

Date: 2026-10-22

Branch: `codex/luna-00a-wasm-build-gate`

## Result

The earlier environment blocker was resolved by running the checks in the official Rust Linux container. The pinned parser builds for `wasm32-unknown-unknown`. Baseline checks for `xezim-core` and Xezim fail in the `getrandom` dependency graph. A disposable browser-feature experiment then made `xezim-core` compile, and exposed an Xezim failure in `libffi-sys`: its Autoconf configuration rejects the `wasm32-unknown-unknown` target.

This is initial compiler evidence for the pinned graph and a narrow patch path. It does **not** establish that Xezim itself is portable. The `libffi-sys` failure blocks validation of Xezim's own sources, C shims, and linker arguments.

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
| `xezim-core --no-default-features` (temporary browser-feature patch) | Passed with target-specific `getrandom` browser features, `RUSTFLAGS=--cfg getrandom_backend="wasm_js"`, and Clang 19 installed in the container. This also shows `zstd-sys` can compile in that configured environment. |
| `xezim --no-default-features --lib` (temporary browser-feature patch) | Failed in `libffi-sys v2.3.0`: Autoconf reports `Invalid configuration 'wasm32-unknown-unknown': OS 'unknown' not recognized`. |

Cargo feature-tree commands confirm the dependency paths:

```sh
cargo tree --manifest-path /src/xezim-core/Cargo.toml --target wasm32-unknown-unknown -e features -i getrandom@0.2.17
cargo tree --manifest-path /src/xezim-core/Cargo.toml --target wasm32-unknown-unknown -e features -i getrandom@0.3.4
```

- `getrandom 0.2.17` is enabled through `rand 0.8` → `rand_core 0.6`.
- `getrandom 0.3.4` is enabled through `ahash 0.8` default `runtime-rng` features.
- Xezim pulls the same paths from its own `rand`/`ahash` dependencies and from xezim-core.

The `--no-default-features` check only removes xezim-core's optional mimalloc allocator; it does not disable these random-number dependencies. Do not enable browser entropy or remove random-seed support without checking simulator semantics and reproducibility requirements.

For the Xezim package, `--no-default-features` also does not disable defaults on its `xezim-core` dependency. An intermediate scratch run therefore tried to cross-compile `libmimalloc-sys` and failed because the wasm target lacked `wchar.h`. Setting `default-features = false` on the browser-only Xezim-to-core dependency exposed the later `libffi-sys` failure. Keep the existing native dependency defaults unchanged.

## Disposable browser-feature experiment

No pinned source checkout or upstream revision was changed. In separate scratch worktrees, both Cargo manifests received these target-specific feature-unifying dependencies:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom_02 = { package = "getrandom", version = "0.2", features = ["js"] }
getrandom_03 = { package = "getrandom", version = "0.3", features = ["wasm_js"] }
```

The scratch Xezim manifest also used local paths to the pinned core/parser checkouts and set `default-features = false` on `xezim-core`. The Rust flag selected getrandom 0.3's browser backend. Clang was installed only in the ephemeral container:

```powershell
$experimentRoot = 'B:\xezim-studio\.codex-rust-luna00\experiment'
$image = 'rust@sha256:a5ab4292e80567dc92f642e2a7d63773ebd695b82b56e8f06dc8d6ecc5600b76'
docker run --rm -v "${experimentRoot}:/work" -e 'RUSTFLAGS=--cfg getrandom_backend="wasm_js"' $image sh -c 'apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends clang lld && rustup target add wasm32-unknown-unknown && cargo check --manifest-path /work/xezim-core/Cargo.toml --target wasm32-unknown-unknown --no-default-features'
docker run --rm -v "${experimentRoot}:/work" -e 'RUSTFLAGS=--cfg getrandom_backend="wasm_js"' $image sh -c 'apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends clang lld && rustup target add wasm32-unknown-unknown && cargo check --manifest-path /work/xezim/Cargo.toml --target wasm32-unknown-unknown --no-default-features --lib'
```

The scratch experiment confirms the two getrandom paths can be configured for browser WASM and that core's zstd path compiles with Clang available. The Xezim gate next requires native FFI dependencies to be target-gated. After that, validate the build script's C shims and linker arguments; this run did not establish their viability.

## Windows host note

Rust was also installed in a task-scoped local directory, but the host had no MSVC build tools. Its `link.exe` resolved to `C:\Program Files\coreutils\bin\link.exe`, which is not the MSVC linker, so host build scripts could not link. The Linux container provided a working Rust host and the proper WASM standard library; the parser check then passed.

## Next patch experiment

In a separate Xezim portability branch, target-gate `libffi`/`libloading` and the C shims/linker arguments while preserving native defaults. Re-run the three checks with the browser getrandom configuration, then test native regression behavior. Keep browser entropy explicit and document its seed behavior.
