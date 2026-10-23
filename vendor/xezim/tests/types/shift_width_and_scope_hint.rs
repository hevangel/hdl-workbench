//! Two defects from a user burst-size testbench, both reference-validated.
//!
//! 1. §11.6.1 / §11.4.10: the left operand of a shift (and of `/ % **`) is
//!    CONTEXT-determined, but the context width is the MAXIMUM of the
//!    surrounding context and the operand's own width — never smaller.
//!    Propagating a narrow LHS width down truncated the operand *before* the
//!    operation, so `logic [4:0] r; r <= (1 << s) >> 3;` with `s == 5`
//!    evaluated `1 << 5` at 5 bits (0) instead of 32 bits (32), yielding 0
//!    instead of 4. (`+ - * & | ^` preserve the low bits either way, which is
//!    why only the shift/divide family exposed it.) Present in BOTH the
//!    bytecode compiler and the interpreter.
//!
//! 2. Scope-hint leak: `resolve_hier_name` installs the parent scope as the
//!    name-resolution hint whenever it resolves a dotted name, and nothing
//!    restored it. After a testbench statement read `u_dut.sig`, the hint
//!    stayed `u_dut`, so the NEXT statement's unqualified names resolved into
//!    the DUT — `sel = 3'd4;` wrote `u_dut.sel` (immediately overwritten by
//!    the port's continuous assign) and the stimulus silently stopped
//!    reaching the design.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A shift whose result is assigned to a NARROWER target must still evaluate
/// the shift at the operand's own width.
#[test]
fn shift_operand_is_not_narrowed_by_target_width() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [2:0] s = 3'd5;
  logic [4:0] nba, blk;
  int init_ctx;
  always #5 clk = ~clk;
  always_ff @(posedge clk) nba <= (1 << s) >> 3;   // bytecode path
  always @(posedge clk) begin                       // interpreter path
    blk = (1 << s) >> 3;
    if (s == 3'd5) blk = blk;                       // defeat trivial compile
  end
  initial begin
    init_ctx = (1 << s) >> 3;
    #26;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "nba"), 4, "nonblocking, narrow target");
    assert_eq!(u(&sim, "blk"), 4, "blocking in an always block");
    assert_eq!(u(&sim, "init_ctx"), 4, "initial block");
}

/// `/` and `%` are equally sensitive to a narrowed left operand.
#[test]
fn divide_operand_is_not_narrowed() {
    let src = r#"
module tb;
  logic [4:0] q, r;
  initial begin
    q = 100 / 7;    // 14
    r = 100 % 9;    // 1
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "q"), 14);
    assert_eq!(u(&sim, "r"), 1);
}

/// Reading a hierarchical name must not redirect the process's later
/// unqualified writes into the referenced scope.
#[test]
fn hierarchical_read_does_not_leak_scope() {
    let src = r#"
module dutm (input logic clk, input logic [2:0] sel, output logic [4:0] lpb);
  always_ff @(posedge clk) lpb <= (1 << sel) >> 3;
endmodule
module tb;
  logic clk = 0;
  logic [2:0] sel = 0;
  wire [4:0] lpb;
  int cap, after_read, sel_seen;
  dutm u_dut(.clk(clk), .sel(sel), .lpb(lpb));
  always #5 clk = ~clk;
  initial begin
    sel = 3'd3;
    repeat (2) @(posedge clk); #1;
    cap = u_dut.lpb;          // hierarchical read: used to leave hint = u_dut
    sel = 3'd4;               // must still write THIS module's sel
    repeat (2) @(posedge clk); #1;
    after_read = u_dut.lpb;
    sel_seen = u_dut.sel;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "cap"), 1, "(1<<3)>>3");
    assert_eq!(u(&sim, "sel_seen"), 4, "stimulus must still reach the DUT");
    assert_eq!(u(&sim, "after_read"), 2, "(1<<4)>>3");
}
