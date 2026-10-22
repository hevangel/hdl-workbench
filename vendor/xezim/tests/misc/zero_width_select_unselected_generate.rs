//! §27.5/§11.5.1: the zero-width indexed part-select lint must not look into
//! a generate branch that is not elaborated. scr1's CSR unit selects
//! `x[(XLEN-1) -: WR_BITS]` only in the branch taken when `WR_BITS != 0`;
//! with `WR_BITS == 0` the lint still evaluated that dead branch and
//! rejected the core (sv-tests `scr1`).

use xezim::simulate;

#[test]
fn zero_width_select_in_unselected_branch_is_ignored() {
    let src = r#"
module m;
  localparam int WR = 0;
  logic [31:0] d = 32'hdead_beef;
  logic [31:0] q;
  generate
    if (WR == 0) begin : ro
      assign q = d;
    end else begin : rw
      assign q = {d[31 -: WR], d[31-WR:0]};
    end
  endgenerate
  initial #1 $display("Z|%h", q);
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "Z|deadbeef"));
}

#[test]
fn zero_width_select_in_selected_branch_is_still_rejected() {
    let src = r#"
module m;
  localparam int WR = 0;
  logic [31:0] d, q;
  generate
    if (WR == 0) begin : ro
      assign q = d[31 -: WR];
    end
  endgenerate
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}
