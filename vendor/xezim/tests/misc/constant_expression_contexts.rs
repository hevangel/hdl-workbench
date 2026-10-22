//! Constant expressions (§6.20, §7.4, §11.5.1, §10.3): a declaration's range,
//! a parameter's value, a part-select's bounds, an indexed part-select's
//! width and the selects of a continuous-assignment target cannot read a
//! variable, net or port. Each rejected case is also rejected by the reference
//! simulator, and the legal module prints the same line there.

use xezim::simulate;

fn rejected(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("accepted an illegal design:\n{src}"),
        Err(e) => e,
    }
}

#[test]
fn variables_are_not_constants() {
    let e = rejected("module top(input [2:0] N, input [7:N] In); endmodule");
    assert!(e.contains("'N' is a variable or net"), "{e}");
    rejected("module top; integer v; parameter P = v; endmodule");
    rejected("module top; wire [7:0] a [7:0]; wire [2:0] n; assign a[n][0] = 1'b1; endmodule");
    rejected("module top; logic [7:0] x; int i; initial begin i = 3; x[i:0] = 0; end endmodule");
    rejected(
        "module top; logic [7:0] x; int w; logic [3:0] y;\n\
         initial begin w = 2; y = x[0 +: w]; end endmodule",
    );
    rejected(
        "module top; function automatic int f(int n); logic [n-1:0] t; t = 0; return t;\n\
         endfunction initial $display(f(3)); endmodule",
    );
}

#[test]
fn constants_in_legal_contexts() {
    let src = r#"
module top;
  parameter W = 8;
  localparam L = W * 2;
  logic [W-1:0] a;
  logic [L-1:0] b;
  logic [7:0] m [W];
  logic [W-1:0] c;
  logic [7:0] vv;
  int i;
  genvar g;
  for (g = 0; g < 2; g++) begin : gb
    logic [g:0] r;
    assign r = '1;
  end
  assign vv[2] = 1'b1;
  initial begin
    i = 2;
    a = 8'hA5;
    b = {a, a};
    c = a[i +: 4];
    m[i] = a[7:4];
    #1 $display("L4|%0d %0d %0d %0d %0d %0d", $bits(b), c, m[2], gb[1].r, $bits(c), vv[2]);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal constant uses must run");
    assert!(
        sim.output.iter().any(|o| o.message == "L4|16 9 10 3 8 1"),
        "{:?}",
        sim.output
    );
}
