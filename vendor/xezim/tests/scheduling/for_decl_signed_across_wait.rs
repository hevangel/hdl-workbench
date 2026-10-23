//! IEEE 1800-2023 §12.7.1: a variable declared in a `for` header has the
//! header's data type, so its signedness is the declared one (§6.11.1) and
//! the loop condition compares with it (§11.8.1):
//! `for (int i = 2; i >= 0; i--)` runs three times and
//! `for (bit [3:0] i = 2; i > -1; i--)` never runs.
//!
//! A loop whose body blocks (`#10`, `@(posedge clk)`, a blocking task call)
//! runs on the suspend-aware path, which lowers it to
//! `while (cond) begin body; step; end`; a loop whose body does not block
//! runs synchronously. Both declare the variable the same way. The
//! synchronous loop used to keep the initializer's signedness for a header
//! type that is unsigned, or that is a typedef of a signed type: the signed
//! literal `2` made `bit [3:0] i` signed, so `i > -1` held, and `2'd2` made a
//! typedef of `int` unsigned, so `i >= 0` never failed.
//!
//! Every count here matches Verilator 5.038 (`--binary --timing`). Each loop
//! stops itself after 10 passes, so a regression fails instead of hanging.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The reported shapes: a count-down `int` loop with a delay in its body.
#[test]
fn count_down_int_across_delay() {
    let src = r#"
module tb;
  int ge = 0, gt = 0, sub = 0, dec = 0, last = 0, neg = 0;
  initial begin
    for (int i = 2; i >= 0; i--) begin ge++; if (ge > 9) break; #10; end
    for (int i = 2; i > -1; i--) begin gt++; if (gt > 9) break; #10; end
    for (int i = 2; i >= 0; i = i - 1) begin sub++; if (sub > 9) break; #10; end
    for (int i = 2; i >= 0; --i) begin dec++; if (dec > 9) break; #10; end
    // The value read back after the delay is signed as well.
    for (int i = 1; i >= -1; i--) begin #10; last = i; if (i < 0) neg++; end
  end
endmodule
"#;
    let sim = simulate(src, 100_000).expect("simulate failed");
    assert_eq!(u(&sim, "ge"), 3, "for (int i = 2; i >= 0; i--) with #10");
    assert_eq!(u(&sim, "gt"), 3, "for (int i = 2; i > -1; i--) with #10");
    assert_eq!(u(&sim, "sub"), 3, "step i = i - 1");
    assert_eq!(u(&sim, "dec"), 3, "step --i");
    assert_eq!(u(&sim, "last"), 0xFFFF_FFFF, "last pass sees i == -1");
    assert_eq!(u(&sim, "neg"), 1, "i < 0 holds on the last pass");
}

/// Every signed header type, including a typedef of `int` and two
/// declarators in one header.
#[test]
fn signed_header_types_across_delay() {
    let src = r#"
module tb;
  typedef int sint_t;
  int b = 0, s = 0, l = 0, g = 0, si = 0, lv = 0, td = 0, tu = 0, two = 0;
  initial begin
    for (byte i = 2; i >= 0; i--) begin b++; if (b > 9) break; #1; end
    for (shortint i = 2; i >= 0; i--) begin s++; if (s > 9) break; #1; end
    for (longint i = 2; i >= 0; i--) begin l++; if (l > 9) break; #1; end
    for (integer i = 2; i >= 0; i--) begin g++; if (g > 9) break; #1; end
    for (int signed i = 2; i >= 0; i--) begin si++; if (si > 9) break; #1; end
    for (logic signed [7:0] i = 2; i >= 0; i--) begin lv++; if (lv > 9) break; #1; end
    for (sint_t i = 2; i >= 0; i--) begin td++; if (td > 9) break; #1; end
    for (sint_t i = 2'd2; i >= 0; i--) begin tu++; if (tu > 9) break; #1; end
    for (int i = 2, j = 0; i >= 0; i--, j++) begin two++; if (two > 9) break; #1; end
  end
endmodule
"#;
    let sim = simulate(src, 100_000).expect("simulate failed");
    assert_eq!(u(&sim, "b"), 3, "byte");
    assert_eq!(u(&sim, "s"), 3, "shortint");
    assert_eq!(u(&sim, "l"), 3, "longint");
    assert_eq!(u(&sim, "g"), 3, "integer");
    assert_eq!(u(&sim, "si"), 3, "int signed");
    assert_eq!(u(&sim, "lv"), 3, "logic signed [7:0]");
    assert_eq!(u(&sim, "td"), 3, "typedef int");
    assert_eq!(u(&sim, "tu"), 3, "typedef int, unsigned initializer");
    assert_eq!(u(&sim, "two"), 3, "two declarators");
}

/// Event controls, tasks, class methods and `always` blocks take the same
/// suspend-aware path.
#[test]
fn blocking_bodies_in_tasks_classes_and_always() {
    let src = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  int ev = 0, at = 0, st = 0, ct = 0, call = 0, alw = 0, alw_done = 0;
  task automatic t_auto(output int n);
    n = 0;
    for (int i = 2; i >= 0; i--) begin n++; if (n > 9) break; #1; end
  endtask
  task t_static(output int n);
    n = 0;
    for (int i = 2; i >= 0; i--) begin n++; if (n > 9) break; #1; end
  endtask
  task automatic tick(); @(posedge clk); endtask
  class C;
    task run(output int n);
      n = 0;
      for (int i = 2; i >= 0; i--) begin n++; if (n > 9) break; #1; end
    endtask
  endclass
  always begin
    for (int i = 2; i > -1; i--) begin alw++; if (alw > 9) break; @(posedge clk); end
    alw_done = 1;
    @(negedge alw_done);
  end
  initial begin
    C c = new;
    for (int i = 2; i >= 0; i--) begin ev++; if (ev > 9) break; @(posedge clk); end
    t_auto(at);
    t_static(st);
    c.run(ct);
    for (int i = 2; i >= 0; i--) begin call++; if (call > 9) break; tick(); end
  end
endmodule
"#;
    let sim = simulate(src, 100_000).expect("simulate failed");
    assert_eq!(u(&sim, "ev"), 3, "@(posedge clk) body");
    assert_eq!(u(&sim, "at"), 3, "automatic task");
    assert_eq!(u(&sim, "st"), 3, "static task");
    assert_eq!(u(&sim, "ct"), 3, "class task");
    assert_eq!(u(&sim, "call"), 3, "body blocks in a task call");
    assert_eq!(u(&sim, "alw"), 3, "always block");
    assert_eq!(u(&sim, "alw_done"), 1, "always block leaves the loop");
}

/// The header type, not the initializer, decides the signedness, with or
/// without a delay in the body, in initial blocks, functions and class
/// methods. An unsigned `i > -1` compares unsigned and never holds; a typedef
/// of `int` stays signed under an unsigned initializer.
#[test]
fn header_type_decides_signedness() {
    let src = r#"
module tb;
  typedef bit [3:0] u4_t;
  typedef int sint_t;
  int bd = 0, bs = 0, ud = 0, us = 0, iu = 0, lg = 0, bu = 0, m1 = 0;
  int ui = 0, sd = 0, ts = 0, fu = 0, ft = 0, cu = 0, cs = 0;
  function automatic int f_u();
    int n = 0;
    for (bit [3:0] i = 2; i > -1; i--) begin n++; if (n > 9) break; end
    return n;
  endfunction
  function automatic int f_t();
    int n = 0;
    for (sint_t i = 2'd2; i >= 0; i--) begin n++; if (n > 9) break; end
    return n;
  endfunction
  class C;
    function int run_u();
      int n = 0;
      for (bit [3:0] i = 2; i > -1; i--) begin n++; if (n > 9) break; end
      return n;
    endfunction
    function int run_t();
      int n = 0;
      for (sint_t i = 2'd2; i >= 0; i--) begin n++; if (n > 9) break; end
      return n;
    endfunction
  endclass
  initial begin
    C c = new;
    for (bit [3:0] i = 2; i > -1; i--) begin bd++; if (bd > 9) break; #1; end
    for (bit [3:0] i = 2; i > -1; i--) begin bs++; if (bs > 9) break; end
    for (u4_t i = 2; i > -1; i--) begin ud++; if (ud > 9) break; #1; end
    for (u4_t i = 2; i > -1; i--) begin us++; if (us > 9) break; end
    for (int unsigned i = 2; i > -1; i--) begin iu++; if (iu > 9) break; end
    for (logic [7:0] i = 2; i > -1; i--) begin lg++; if (lg > 9) break; end
    for (byte unsigned i = 2; i > -1; i--) begin bu++; if (bu > 9) break; end
    // -1 fitted to an unsigned 4 bits is 15.
    for (bit [3:0] i = -1; i == 15; i--) begin m1++; if (m1 > 9) break; end
    // 2, 1, 0, then 4294967295 < 5 fails.
    for (int unsigned i = 2; i < 5; i--) begin ui++; if (ui > 9) break; #1; end
    for (sint_t i = 2; i >= 0; i--) begin sd++; if (sd > 9) break; end
    for (sint_t i = 2'd2; i >= 0; i--) begin ts++; if (ts > 9) break; end
    fu = f_u();
    ft = f_t();
    cu = c.run_u();
    cs = c.run_t();
  end
endmodule
"#;
    let sim = simulate(src, 100_000).expect("simulate failed");
    assert_eq!(u(&sim, "bd"), 0, "bit [3:0] > -1 with #1");
    assert_eq!(u(&sim, "bs"), 0, "bit [3:0] > -1 without a delay");
    assert_eq!(u(&sim, "ud"), 0, "typedef bit [3:0] > -1 with #1");
    assert_eq!(u(&sim, "us"), 0, "typedef bit [3:0] > -1 without a delay");
    assert_eq!(u(&sim, "iu"), 0, "int unsigned > -1");
    assert_eq!(u(&sim, "lg"), 0, "logic [7:0] > -1");
    assert_eq!(u(&sim, "bu"), 0, "byte unsigned > -1");
    assert_eq!(u(&sim, "m1"), 1, "bit [3:0] i = -1 reads 15");
    assert_eq!(u(&sim, "ui"), 3, "int unsigned wraps and exits");
    assert_eq!(u(&sim, "sd"), 3, "typedef int without a delay");
    assert_eq!(u(&sim, "ts"), 3, "typedef int, unsigned initializer");
    assert_eq!(u(&sim, "fu"), 0, "bit [3:0] in a function");
    assert_eq!(u(&sim, "ft"), 3, "typedef int in a function");
    assert_eq!(u(&sim, "cu"), 0, "bit [3:0] in a class method");
    assert_eq!(u(&sim, "cs"), 3, "typedef int in a class method");
}
