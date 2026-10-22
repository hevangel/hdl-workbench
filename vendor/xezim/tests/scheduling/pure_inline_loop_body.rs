//! Bytecode pure-function inlining must never defer part of an inlined body to
//! the AST interpreter.
//!
//! Inlining binds a function's formals, locals and return variable to bytecode
//! REGISTERS. `emit_fallback` defers a statement it cannot compile to the AST
//! interpreter, which resolves names through the signal tables — where those
//! registers do not exist. A deferred statement inside an inlined body
//! therefore reads and writes the wrong storage and its effect is silently
//! lost.
//!
//! The shape that exposed it: a pure helper that accumulates in a `for` loop.
//! Loops are not compiled inside an inlined body, so the loop was deferred, the
//! register-backed accumulator never updated, and the function returned its
//! INITIAL value. No fallback was counted and no diagnostic printed — the
//! answer was simply wrong, and only in the bytecode path, so the identical
//! call from an `initial` block returned the right value.
//!
//! This is the failure mode inlining must be held to: if any statement will not
//! compile, the whole inline has to fail so the ordinary call path is used.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A pure function accumulating in a loop must return the same value from the
/// bytecode path (`always_ff`) as from the interpreter (`initial`).
#[test]
fn pure_function_with_loop_returns_same_in_both_paths() {
    let src = r#"
module tb;
  localparam int DEPTH = 8;
  localparam int W = 32;
  logic clk = 0;
  logic [W-1:0] r_call, r_direct;
  always #5 clk = ~clk;
  function automatic logic [W-1:0] tail_bias();
    logic [W-1:0] acc;
    acc = '0;
    for (int i = 1; i < DEPTH; i++) acc = acc + i[W-1:0];
    tail_bias = acc;
  endfunction
  always_ff @(posedge clk) r_call <= tail_bias();
  initial begin
    #40;
    r_direct = tail_bias();
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    // 1+2+..+7
    assert_eq!(u(&sim, "r_direct"), 28, "interpreter path");
    assert_eq!(u(&sim, "r_call"), 28, "bytecode path (inlined)");
}

/// The loop must run for a function taking arguments too — the formals are
/// register-bound exactly like the locals.
#[test]
fn pure_function_with_args_and_loop() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [31:0] popc, weighted;
  always #5 clk = ~clk;
  function automatic logic [31:0] popcount(input logic [31:0] v);
    logic [31:0] n;
    n = 0;
    for (int i = 0; i < 32; i++) if (v[i]) n = n + 1;
    popcount = n;
  endfunction
  function automatic logic [31:0] weight(input logic [31:0] v, input logic [31:0] k);
    logic [31:0] a;
    a = 0;
    for (int i = 0; i < 4; i++) a = a + (v >> (8*i)) * k;
    weight = a;
  endfunction
  always_ff @(posedge clk) begin
    popc     <= popcount(32'hF0F0_000F);
    weighted <= weight(32'h0000_0102, 32'd3);
  end
  initial #40 $finish;
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "popc"), 12, "8 + 4 set bits");
    // v>>0 = 0x0102, v>>8 = 0x01, v>>16 = 0, v>>24 = 0 -> (258 + 1) * 3
    assert_eq!(u(&sim, "weighted"), (258 + 1) * 3);
}

/// A loop nested inside `if`/`case` within the body is the same hazard.
#[test]
fn pure_function_loop_nested_in_control_flow() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [31:0] a, b;
  always #5 clk = ~clk;
  function automatic logic [31:0] cond_sum(input logic sel);
    logic [31:0] acc;
    acc = 0;
    if (sel) begin
      for (int i = 1; i <= 4; i++) acc = acc + i;
    end else begin
      for (int i = 1; i <= 3; i++) acc = acc + (i * 10);
    end
    cond_sum = acc;
  endfunction
  always_ff @(posedge clk) begin
    a <= cond_sum(1'b1);
    b <= cond_sum(1'b0);
  end
  initial #40 $finish;
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 10, "1+2+3+4");
    assert_eq!(u(&sim, "b"), 60, "10+20+30");
}

/// A `while` loop, to confirm the guard is not specific to `for`.
#[test]
fn pure_function_with_while_loop() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [31:0] r;
  always #5 clk = ~clk;
  function automatic logic [31:0] countdown(input logic [31:0] n);
    logic [31:0] acc;
    acc = 0;
    while (n > 0) begin
      acc = acc + n;
      n = n - 1;
    end
    countdown = acc;
  endfunction
  always_ff @(posedge clk) r <= countdown(32'd5);
  initial #40 $finish;
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "r"), 15, "5+4+3+2+1");
}

/// Loop-free pure functions must STILL inline and still be correct — the fix
/// must not disable inlining wholesale.
#[test]
fn loop_free_pure_function_still_correct() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [31:0] s;
  always #5 clk = ~clk;
  function automatic logic [31:0] lfsr32(input logic [31:0] v);
    lfsr32 = {v[30:0], v[31] ^ v[21] ^ v[1] ^ v[0]};
  endfunction
  always_ff @(posedge clk) s <= lfsr32(32'h89AB_CDEF);
  initial #40 $finish;
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    // {0x09ABCDEF<<1 bits}: top bit dropped, new lsb = v31^v21^v1^v0
    let v: u32 = 0x89AB_CDEF;
    let nb = ((v >> 31) ^ (v >> 21) ^ (v >> 1) ^ v) & 1;
    let want = ((v << 1) | nb) as u64;
    assert_eq!(u(&sim, "s"), want);
}
