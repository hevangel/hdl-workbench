//! §27.4: a generate loop's index is a genvar, declared with `genvar` or in
//! the loop header (which may reuse a module variable's name). A module-level
//! `integer` cannot be one. The reference
//! simulator rejects that loop and runs the legal module with the same
//! output.

use xezim::simulate;

#[test]
fn integer_generate_loop_index() {
    let src = r#"
module top;
  integer i = 0;
  generate
    for (i = 0; i < 4; i = i + 1) begin : U
      reg [1:0] a = i;
    end
  endgenerate
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn genvar_generate_loop_index() {
    let src = r#"
module test;
  genvar i;
  integer k, j;
  for (i = 0; i < 2; i++) begin : g
    wire w = i;
  end
  for (genvar j = 0; j < 2; j++) begin : h
  end
  initial begin k = 3; j = 5; #1 $display("G|%0d %0d %b", k, j, g[1].w); end
endmodule
"#;
    let sim = simulate(src, 10).expect("genvar loops are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "G|3 5 1"),
        "{:?}",
        sim.output
    );
}
