# Third-party notices

## Browser C/C++ compiler

The IDE includes `browsercc` 0.1.1 by Daniel Bertalan. Its MIT license is
available at [browsercc-MIT.txt](./browsercc-MIT.txt). Source:
[BertalanD/browsercc](https://github.com/BertalanD/browsercc).

The browsercc 0.1.1 build script uses LLVM `llvmorg-20.1.2`, licensed under
Apache-2.0 with the LLVM exception. See the pinned
[LLVM license](https://github.com/llvm/llvm-project/blob/llvmorg-20.1.2/llvm/LICENSE.TXT).
It also downloads the WASI SDK 25.0 sysroot; the SDK repository uses
Apache-2.0. See the [WASI SDK license](https://github.com/WebAssembly/wasi-sdk/blob/main/LICENSE)
and [WASI SDK source](https://github.com/WebAssembly/wasi-sdk).

The npm package's bundled sysroot does not expose a complete component-level
notice inventory. Some bundled compiler/runtime/sysroot components may have
additional license terms. This notice identifies the compiler sources and
principal licenses but is not a completed third-party audit.
