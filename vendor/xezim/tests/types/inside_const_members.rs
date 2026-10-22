//! §11.4.13 set membership over CONSTANT members compiles to an Eq/Or chain.
//!
//! Every member must be a compile-time constant with no x/z bits — exactly
//! the condition under which `==?` degenerates to `==`, so the compiled form
//! is semantically identical to the interpreter (verified against the
//! reference simulator, X cases included: an x in the OPERAND propagates as
//! Eq's x through the OR chain — x|1 is 1, x|0 is x). Ranges and wildcard
//! members stay on the interpreter. ibex evaluates ~2.6 such expressions per
//! cycle in its decoder and CSR logic.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
package pkg;
  parameter logic [1:0] OPW = 2'd1, OPS = 2'd2, OPC = 2'd3;
endpackage
module top;
  import pkg::*;
  logic [1:0] op;
  logic hit;
  logic [7:0] v;
  logic inr;
  always_comb hit = (op inside {OPW, OPS, OPC});
  always_comb inr = (v inside {8'h10, 8'h20, [8'h30:8'h40]});
  initial begin
    op = 2'd0; v = 8'h35;
    #1 $display("NOTE: A %b %b", hit, inr);
    op = 2'd2; v = 8'h20;
    #1 $display("NOTE: B %b %b", hit, inr);
    op = 2'bx0; v = 8'h99;
    #1 $display("NOTE: C %b %b", hit, inr);
    op = 2'bx1;
    #1 $display("NOTE: D %b", hit);
    $finish;
  end
endmodule
"#;

/// Reference-verified, including x-in-operand propagation (C: x0 matches
/// nothing definitely and 10 exactly-mismatches only on defined bits -> x;
/// D: x1 gives ==01:x, ==10:0, ==11:x -> x).
#[test]
fn inside_const_members_matches_reference() {
    assert_eq!(
        notes(SRC),
        vec!["NOTE: A 0 1", "NOTE: B 1 1", "NOTE: C x 0", "NOTE: D x",]
    );
}
