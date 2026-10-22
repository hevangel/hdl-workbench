//! §7.4.1 — READS of bit-selects and indexed part-selects on a NON-ZERO-BASED
//! packed vector (`logic [3:1] w`, `logic [1:1] h`). Reference-validated.
//!
//! Mirror of `nonzero_based_vector_writes`. Every `BitSelect*` / `RangeSelect`
//! instruction addresses PHYSICAL bits, but SV indices are DECLARED indices —
//! `logic [3:1] w` keeps declared bit 1 at physical offset 0. The write path
//! rebased through `emit_rebased_index`; three read paths did not:
//!
//!   * the bit-select read emission — `h[1]` on a `logic [1:1]` selected
//!     physical bit 1 of a ONE-bit signal and evaluated to x;
//!   * the indexed part-select — `w[1 +: 2]` read declared 3:2 instead of
//!     2:1, and `w[3 -: 2]` ran off the top of the signal and returned x;
//!   * `try_resolve_bit_ref`, the fused-gate fast path, which bypasses the
//!     bytecode compiler entirely, so `assign y = w[1]` read declared bit 2.
//!
//! Two things kept this hidden. `$display("%b", w[1])` goes through the AST
//! interpreter, which rebases correctly — so a procedural probe printed the
//! right value while the same expression in an `assign` did not. And the
//! fused path disagreed with the compiled one only on the LOW indices: `w[3]`
//! failed the fused range check, fell through to bytecode, and came out
//! right, so spot-checking the top bit passed.
//!
//! Field shape: a `logic [1:1] flag` register whose value feeds
//! `state[1] <= ~flag[1]` inside an `always_ff` — the read returned x, the
//! inversion produced 1 instead of 0, and the state machine took the wrong
//! branch.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z, expected a defined value", n))
}

/// Every read form against `logic [3:1] w = 3'b101` (declared w[3]=1, w[2]=0,
/// w[1]=1) and an ascending `logic [1:3] a`. All results are produced by
/// continuous assigns so they compile to bytecode — reading them with
/// `$display` would take the AST path and hide the bug.
const SRC_FORMS: &str = r#"
module tb;
  logic [3:1] w;
  logic [1:3] a;
  logic [7:4] n;
  int i;

  logic r_w1, r_w2, r_w3, r_wi;
  logic [1:0] r_range, r_up, r_down;
  logic r_a1, r_a3;
  logic r_n4, r_n7;
  logic [1:0] r_cat;

  assign r_w1    = w[1];
  assign r_w2    = w[2];
  assign r_w3    = w[3];
  assign r_wi    = w[i];
  assign r_range = w[2:1];
  assign r_up    = w[1 +: 2];
  assign r_down  = w[3 -: 2];
  assign r_a1    = a[1];
  assign r_a3    = a[3];
  assign r_n4    = n[4];
  assign r_n7    = n[7];
  assign r_cat   = {w[3], w[1]};

  initial begin
    i = 1;
    w = 3'b101;
    a = 3'b101;
    n = 4'b1000;   // declared n[7]=1, n[6]=n[5]=n[4]=0
    #1;
  end
endmodule
"#;

#[test]
fn bit_select_reads_use_declared_indices() {
    let sim = simulate(SRC_FORMS, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_w1"), 1, "w[1] of 3'b101 on logic [3:1]");
    assert_eq!(
        u(&sim, "r_w2"),
        0,
        "w[2] — the fused-gate path read declared bit 3 here"
    );
    assert_eq!(u(&sim, "r_w3"), 1, "w[3]");
    assert_eq!(u(&sim, "r_wi"), 1, "w[i] with i=1 (dynamic index)");
    assert_eq!(u(&sim, "r_cat"), 0b11, "{{w[3], w[1]}}");
    // n[7] and n[4] must differ, or a rebase error is invisible.
    assert_eq!(u(&sim, "r_n7"), 1, "n[7] on logic [7:4] = 4'b1000");
    assert_eq!(
        u(&sim, "r_n4"),
        0,
        "n[4] — equal values here would hide a swap"
    );
}

#[test]
fn indexed_part_select_reads_rebase_their_base() {
    let sim = simulate(SRC_FORMS, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "r_range"),
        0b01,
        "w[2:1] — the constant range form was always correct"
    );
    assert_eq!(
        u(&sim, "r_up"),
        0b01,
        "w[1 +: 2] must be declared 2:1, not 3:2"
    );
    assert_eq!(
        u(&sim, "r_down"),
        0b10,
        "w[3 -: 2] must be declared 3:2; an unrebased base ran off the top and returned x"
    );
}

#[test]
fn ascending_vector_reads_use_declared_indices() {
    let sim = simulate(SRC_FORMS, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_a1"), 1, "a[1] on ascending logic [1:3] = 3'b101");
    assert_eq!(u(&sim, "r_a3"), 1, "a[3] on ascending logic [1:3]");
}

/// The reported shape: a `logic [1:1]` register read back through an
/// inversion inside a clocked block. Exercises the single-bit case, where the
/// declared index equals the width so an unrebased select is always out of
/// range and yields x.
const SRC_SINGLE_BIT: &str = r#"
module leaf(input logic clk, input logic rst, input logic d,
            output logic o_assign, output logic o_comb, output logic o_ff, output logic o_inv);
  logic [1:1] h;
  always_ff @(posedge clk) begin
    if (rst) h <= 1'b0; else h[1] <= d;
  end
  assign o_assign = h[1];
  always_comb o_comb = h[1];
  always_ff @(posedge clk) o_ff  <= h[1];
  always_ff @(posedge clk) o_inv <= ~h[1];
endmodule

module tb;
  logic clk = 0, rst = 1, d = 0;
  always #5 clk = ~clk;
  logic o_assign, o_comb, o_ff, o_inv;
  logic [1:1] top_h;
  logic t_assign;
  always_ff @(posedge clk) begin
    if (rst) top_h <= 1'b0; else top_h[1] <= d;
  end
  assign t_assign = top_h[1];
  leaf u(.*);
  initial begin
    @(negedge clk); rst = 0; d = 1;
    repeat (3) @(posedge clk);
    @(negedge clk);
  end
endmodule
"#;

#[test]
fn single_bit_nonzero_based_register_reads_back() {
    let sim = simulate(SRC_SINGLE_BIT, 200).expect("simulate failed");
    for (sig, what) in [
        ("o_assign", "continuous assign"),
        ("o_comb", "always_comb"),
        ("o_ff", "always_ff"),
    ] {
        assert_eq!(u(&sim, sig), 1, "h[1] read from a {what} in a submodule");
    }
    assert_eq!(
        u(&sim, "o_inv"),
        0,
        "~h[1]: an x read inverts to 1 and sends a state machine down the wrong branch"
    );
    // Same construct at TOP level — it was broken here too; the earlier
    // top-level probe only looked correct because it printed with $display.
    assert_eq!(
        u(&sim, "t_assign"),
        1,
        "top_h[1] read from a top-level assign"
    );
}
