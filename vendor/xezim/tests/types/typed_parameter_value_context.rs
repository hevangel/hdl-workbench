//! §6.20.2 / §10.7: a parameter declared with a type or a range is ASSIGNED
//! its value, so the declared width is the initializer's context —
//! `localparam int P1 = PA + PB` over 8-bit PA and PB is 300, where the
//! elaborator folded the sum at 8 bits (44). An untyped parameter keeps its
//! value's self-determined width, and a `real` one takes no integral context
//! (both stay 44).
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn typed_parameter_value_is_assignment_context() {
    let src = r#"
module top;
  localparam bit [7:0] PA = 200;
  localparam bit [7:0] PB = 100;
  localparam int P1 = PA + PB;
  localparam bit [15:0] P2 = PA + PB;
  localparam P3 = PA + PB;
  localparam [11:0] P4 = PA * PB;
  localparam int P5 = PA - PB - 150;
  localparam logic [15:0] P6 = (PA + PB) >> 1;
  localparam byte P7 = PA + PB;
  localparam real P8 = PA + PB;
  localparam int P9 = PA > PB ? PA + PB : PB;
  localparam longint P10 = PA * PB * PA;
  localparam int P11 = -PA;
  initial $display("P %0d %0d %0d %0d %0d %0d %0d %0.1f %0d %0d %0d", P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    let want = "P 300 300 44 3616 -50 150 44 44.0 300 4000000 -200";
    assert!(
        o.iter().any(|l| l.trim() == want),
        "missing {want:?} in {o:?}"
    );
}
