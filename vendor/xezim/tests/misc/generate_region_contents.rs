//! §27: a generate block holds module_or_generate items only — no timeunit
//! declaration, no specify block, and no nested `generate` region (§27.3).
//! Each rejected case is also rejected by the reference simulator, and the
//! legal module prints the same line there.

use xezim::simulate;

#[test]
fn illegal_generate_items() {
    for src in [
        "module test #(parameter A = 1);\n\
         generate if (A) begin timeunit 10ns/1ns; end endgenerate endmodule",
        "module test #(parameter A = 1);\n\
         generate if (A) begin specify endspecify end endgenerate endmodule",
        "module top; wire x, y; reg in; genvar i;\n\
         generate for (i=0; i<1; i=i+1) begin assign x = in; end\n\
         generate for (i=0; i<1; i=i+1) begin assign y = in; end endgenerate\n\
         endgenerate endmodule",
    ] {
        assert!(simulate(src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn generate_blocks_with_legal_items() {
    let src = r#"
module top #(parameter A = 1);
  timeunit 1ns; timeprecision 1ps;
  wire [1:0] x; reg in;
  genvar i;
  generate
    for (i = 0; i < 2; i = i + 1) begin : g
      if (A) begin : h
        assign x[i] = in;
      end
    end
  endgenerate
  specify
  endspecify
  initial begin in = 1; #1 $display("G|%b", x); end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal generate must run");
    assert!(
        sim.output.iter().any(|o| o.message == "G|11"),
        "{:?}",
        sim.output
    );
}
