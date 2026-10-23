//! §23.6: in a hierarchical name, the segment after a named `begin`/`fork`
//! block is looked up in that block. `U[0].V.a`, where `a` belongs to the
//! generate block `U` and `V` declares nothing, is an error (ivtest
//! `pr1988302b`; the reference simulator reports "Failed to find 'a' in
//! hierarchical name"). The runtime resolver fell back outward and read the
//! generate block's `a`.

use xezim::simulate;

const DESIGN: &str = r#"
module main;
  generate
    genvar i;
    for (i = 0; i < 4; i = i + 2) begin : U
      reg [1:0] a;
      initial begin : V
        reg [1:0] own;
        a = 2'b0;
        own = i;
      end
    end
  endgenerate
  initial #1 $display("H|%0d", REF);
endmodule
"#;

#[test]
fn name_not_declared_in_the_named_block_is_rejected() {
    assert!(simulate(&DESIGN.replace("REF", "U[2].V.a"), 10).is_err());
}

#[test]
fn names_declared_in_the_block_or_its_parent_scope_are_accepted() {
    assert!(simulate(&DESIGN.replace("REF", "U[2].V.own"), 10).is_ok());
    let sim = simulate(&DESIGN.replace("REF", "U[2].a"), 10).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "H|0"));
}
