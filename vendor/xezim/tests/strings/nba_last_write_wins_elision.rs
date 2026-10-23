//! §10.4.2: when one variable is the target of several nonblocking assignments
//! in the same time step, the LAST one scheduled determines the final value.
//!
//! The NBA queue elides a write whose value already equals the signal's current
//! value — a large win, because flop outputs reload the same value most cycles.
//! That comparison was made against the signal table alone, ignoring any value
//! ALREADY QUEUED for the same signal in this time step. So a later assignment
//! that happened to match the *stale* pre-update value was dropped, and the
//! earlier one survived — exactly inverting last-write-wins.
//!
//! The shape that exposes it is the standard synchronous-reset idiom, where a
//! reset clause deliberately overwrites assignments made earlier in the same
//! always block:
//!
//! ```systemverilog
//! always @(posedge clk) begin
//!   d1 <= in;
//!   d2 <= d1 - 1'b1;
//!   if (rst) {d1, d2} <= '0;   // must win
//! end
//! ```
//!
//! With `d2` already 0 and `d1 - 1'b1` evaluating to 1, the reset's `d2 <= 0`
//! matched the current value, was elided, and the pipeline value survived the
//! reset. Reference-validated.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A concatenation-LHS reset must override every earlier NBA in the block,
/// including the one whose source is an expression reading a reset target.
#[test]
fn concat_reset_overrides_earlier_nba() {
    let src = r#"
module tb;
  logic clk = 0;
  logic rst = 1;
  logic a, b, c, d;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    a <= 1'b1;
    b <= 1'b1;
    c <= a - 1'b1;        // evaluates to 1 while a is held at 0 by reset
    d <= 1'b1;
    if (rst) begin
      {a, b, c, d} <= 4'b0;
    end
  end
  initial #100 $finish;
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    for n in ["a", "b", "c", "d"] {
        assert_eq!(u(&sim, n), 0, "{} escaped the reset", n);
    }
}

/// The same defect with scalar targets and no concatenation: a later NBA whose
/// value equals the CURRENT value must still displace an earlier queued one.
#[test]
fn later_nba_matching_current_value_still_wins() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [3:0] x = 4'd0;
  logic [3:0] y = 4'd7;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    x <= 4'd5;      // would change x
    x <= 4'd0;      // equals the current value — must still win
    y <= 4'd7;      // equals current: no-op
    y <= 4'd9;      // must win
  end
  initial #40 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "x"), 0, "later matching-value NBA must win");
    assert_eq!(u(&sim, "y"), 9, "later differing NBA must win");
}

/// A whole-value NBA after a partial (bit/range) NBA must replace the whole
/// signal, not merge into the queued partial.
#[test]
fn whole_value_nba_after_partial_replaces_it() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [7:0] v = 8'h00;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    v[3:0] <= 4'hF;
    v      <= 8'h00;   // equals current value; must still win over the partial
  end
  initial #40 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "v"),
        0x00,
        "whole-value NBA must override the partial"
    );
}

/// Reset priority through several cycles of real pipeline motion — the
/// end-to-end shape from the reported testbench.
#[test]
fn sync_reset_holds_pipeline_at_zero() {
    let src = r#"
module dut (input logic clk, input logic rst, input logic din,
            output logic [2:0] q);
  logic d1, d2, d3;
  always @(posedge clk) begin
    d1 <= din;
    d2 <= d1;
    d3 <= d2 - 1'b1;
    if (rst) begin
      {d1, d2, d3} <= 'b0;
    end
  end
  assign q = {d1, d2, d3};
endmodule
module tb;
  logic clk = 0, rst = 1, din = 1;
  wire [2:0] q;
  int held, after_release;
  dut u (.clk(clk), .rst(rst), .din(din), .q(q));
  always #5 clk = ~clk;
  initial begin
    repeat (6) @(posedge clk);
    #1 held = q;              // reset held: every stage 0
    rst = 1'b0;
    repeat (4) @(posedge clk);
    #1 after_release = q;     // pipeline filled with din=1
  end
endmodule
"#;
    let sim = simulate(src, 300).expect("simulate failed");
    assert_eq!(u(&sim, "held"), 0, "reset must hold all three stages at 0");
    assert_eq!(
        u(&sim, "after_release"),
        0b110,
        "d1=1 d2=1 d3=(d2-1)=0 once the pipeline has filled"
    );
}
