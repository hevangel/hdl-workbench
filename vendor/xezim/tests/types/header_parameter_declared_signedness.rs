//! §6.20.2: a module-header parameter with an explicit type or range takes
//! that type's signedness. The header path only ever SET the flag, so the
//! signedness of an unsized (signed) initializer literal leaked through:
//! `parameter bit [7:0] PA = 200` read -56, `PA > 100` was false, and every
//! constant built from it (`int'(PA + PB)`) folded as a negative number.
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn header_parameter_keeps_declared_signedness() {
    let src = r#"
typedef int myint5_t;
module top #(parameter bit [7:0] PA = 200, parameter logic [7:0] PC = 8'd200,
             parameter [7:0] PD = 200, parameter int PE = -5, parameter myint5_t PT = -5,
             parameter PU = 200, parameter bit [7:0] PB = 100);
  localparam int L1 = int'(PA + PB);
  localparam int L2 = PA + PB;
  initial begin
    $display("H %0d %0d %0d %0d %0d %0d %0d %0d %0d", PA, PA > 100, PC, PD, PD > 100, PE, PT, PT < 0, PU);
    $display("I %0d %0d %0d", L1, L2, PA + PB);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["H 200 1 200 200 1 -5 -5 1 200", "I 300 300 44"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
