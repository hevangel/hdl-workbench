//! §11.6.1: `/` and `%` are context-determined in BOTH operands — only a shift
//! amount and an exponent are self-determined. The interpreter sized a
//! division from its LEFT operand and the context alone, so `(a + b) / 3`
//! over 8-bit `a = 200`, `b = 100` summed at 8 bits (44 / 3 = 14) although
//! the unsized literal makes the expression 32 bits wide (300 / 3 = 100). The
//! compiled path already took both operands.
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn division_is_sized_by_both_operands() {
    let src = r#"
module top;
  bit [7:0] a = 200, b = 100;
  logic [7:0] q8;
  int i;
  bit signed [7:0] sa = -100;
  initial begin
    q8 = (a + b) / 3;
    i = (a + b) % 7;
    $display("A %0d %0d %0d %0d", (a + b) / 3, (a + b) % 7, q8, i);
    $display("B %0d %0d %0d", 4'((a + b) / 3), (a + b) / 8'd3, (a + b) % 8'd7);
    $display("C %0d %0d", (sa - 100) / 2, (a * 2) / 3);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["A 100 6 100 6", "B 4 14 2", "C -100 133"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
