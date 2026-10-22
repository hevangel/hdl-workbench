# LUNA-11 scalar DPI compiler spike

This spike checks the smallest proposed browser ABI: one C or C++ translation
unit compiled as a standalone `wasm32-unknown-unknown` module, exporting an
`int` function with `int` inputs. The first SystemVerilog acceptance case is:

```systemverilog
module top;
  import "DPI-C" function int add_ints(input int left, input int right);
  initial begin
    if (add_ints(10, 20) != 30) $fatal(1, "DPI result mismatch");
    $display("DPI_ADD=30");
    $finish;
  end
endmodule
```

The module is intended to have no imported host functions and no shared
linear-memory requirement. The WebAssembly JavaScript API can call its
`add_ints` export synchronously. This is a compiler/ABI feasibility fixture;
it does not enable DPI in Xezim yet.

The reproducible builder command uses Clang for code generation and the
`wasm-ld` shipped with the pinned Rust toolchain for linking. It deliberately
uses `-nostdlib`; C++ standard-library support and non-scalar DPI types are
separate acceptance stages.

```sh
WASM_LD="$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/wasm-ld"
clang --target=wasm32-unknown-unknown -c -O2 spikes/luna11-dpi/scalar_dpi.c -o /tmp/scalar_dpi.o
"$WASM_LD" --no-entry --export=add_ints /tmp/scalar_dpi.o -o /tmp/scalar_dpi.wasm
clang --target=wasm32-unknown-unknown -fno-exceptions -fno-rtti -c -O2 spikes/luna11-dpi/scalar_dpi.cpp -o /tmp/scalar_dpi_cpp.o
"$WASM_LD" --no-entry --export=add_ints_cpp /tmp/scalar_dpi_cpp.o -o /tmp/scalar_dpi_cpp.wasm
```

Use a real browser to verify both the export signature and invocation before
the profile is marked supported. Never replace a missing export or an ABI
error with a zero return value.
