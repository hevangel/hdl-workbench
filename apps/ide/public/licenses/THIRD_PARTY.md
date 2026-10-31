# Third-party notices

## Browser C/C++ compiler assets

The IDE ships the `browsercc` 0.1.1 npm artifact. Its exact package integrity
is recorded in `session_history/initial_build/luna-11-dpi-c-cpp-report.md`. The generated build assets
identified below are from that locked package; their SHA-256 values provide a
check against accidental asset changes.

| Component | Provenance | License and notice |
| --- | --- | --- |
| `browsercc` JavaScript and browser glue | npm `browsercc@0.1.1` | [MIT license](browsercc-MIT.txt); [upstream source](https://github.com/BertalanD/browsercc) |
| Clang WebAssembly compiler | LLVM `llvmorg-20.1.2`; `clang.wasm`, SHA-256 `d574339640a9764544f405d4b5ae79ef7d05e49725b416fba74f5b15c00f8ee5` | [LLVM license and exception](LLVM-LICENSE-20.1.2.txt); [pinned source license](https://github.com/llvm/llvm-project/blob/llvmorg-20.1.2/llvm/LICENSE.TXT) |
| LLD WebAssembly linker | LLVM `llvmorg-20.1.2`; `lld.wasm`, SHA-256 `810bb3be4eec8ad6bae60ceb7355c07bf7de8a54b5c8acb4d7a80613bd43c4bf` | [LLVM license and exception](LLVM-LICENSE-20.1.2.txt); [pinned source license](https://github.com/llvm/llvm-project/blob/llvmorg-20.1.2/llvm/LICENSE.TXT) |
| WASI SDK sysroot and WASI C runtime | WASI SDK `wasi-sdk-25`, including `sysroot.tar`, SHA-256 `dbed83f93f0fa695f36041b82a95a8cd78f106a3f49d9cd860e57ae1ac823318` | [WASI SDK license](WASI-SDK-LICENSE-25.0.txt); [pinned SDK source](https://github.com/WebAssembly/wasi-sdk/tree/wasi-sdk-25) |
| wasi-libc and bundled runtime components | wasi-libc `wasi-sdk-25` submodule in the SDK release | [wasi-libc license summary](wasi-libc-LICENSE-25.0.txt), [Apache-2.0](wasi-libc-LICENSE-APACHE.txt), [Apache-2.0 with LLVM exception](wasi-libc-LICENSE-APACHE-LLVM.txt), and [MIT](wasi-libc-LICENSE-MIT.txt); [pinned source](https://github.com/WebAssembly/wasi-libc/tree/wasi-sdk-25) |
| Cloudlibc | Included by wasi-libc | [BSD-2-Clause license](cloudlibc-LICENSE-BSD-2-Clause.txt) |
| musl | Included by wasi-libc | [musl copyright and MIT terms](musl-COPYRIGHT.txt) |
| musl-fts | Included by wasi-libc | [BSD-3-Clause terms](musl-fts-COPYING-BSD-3-Clause.txt) |
| Emscripten runtime used to build `clang.wasm` and `lld.wasm` | The browsercc build container installs the mutable `latest` emsdk channel; the npm artifact is pinned, but its exact Emscripten revision is not recorded upstream | [Emscripten license](Emscripten-LICENSE.txt); [upstream source](https://github.com/emscripten-core/emscripten) |

The wasi-libc notice identifies additional allocator components: dlmalloc under
CC0, and emmalloc under MIT. Its original source notices are referenced by the
[pinned wasi-libc license summary](https://github.com/WebAssembly/wasi-libc/blob/wasi-sdk-25/LICENSE).
LLVM's license also states that its source tree contains separately licensed
third-party software; see the bundled LLVM license for its notice policy.

The browsercc build script downloads the WASI sysroot, then removes its
`share/` directory and version file when creating `sysroot.tar`. Inspection of
the shipped tar found 1,594 entries and no license/notice files. The package
therefore does not carry a component manifest for every item in the sysroot.
The browsercc build container also installs Emscripten from `latest`, so that
toolchain revision cannot be reconstructed from the package metadata alone.
The npm lock integrity and asset hashes pin the bytes used by this app, but do
not make the upstream build recipe reproducible.

This is a technical asset and notice inventory, not a qualified legal review.
The remaining per-component inventory and the combined application's
distribution position require the release review tracked in
`session_history/initial_build/surfer-distribution-review.md` in the source repository.
