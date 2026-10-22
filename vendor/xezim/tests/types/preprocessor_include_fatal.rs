//! A failed `include` must FAIL the run, not warn and continue.
//!
//! Continuing used to silently drop the include's declarations, and the
//! damage surfaced far away: names used only as port actuals became silent
//! 1-bit implicit nets, so a DUT port read a truncated connection and a whole
//! testbench checked garbage. The reference tooling hard-errors here.
//!
//! The implicit nets themselves are LEGAL (§6.10) and stay 1-bit scalar —
//! reference-validated: the port side reads its declared width while the
//! outer net is 1 bit. The warning now says what the likely root is.

use xezim::simulate;

#[test]
fn missing_include_is_a_hard_error() {
    let src = r#"
module tb;
`include "definitely_not_a_real_file_xyz.svh"
  initial $finish;
endmodule
"#;
    let result = simulate(src, 50);
    match result {
        Err(e) => assert!(
            e.contains("include"),
            "error must name the include failure, got: {}",
            e
        ),
        Ok(_) => panic!("a missing `include must fail the run"),
    }
}

#[test]
fn undeclared_port_actual_is_a_scalar_implicit_net() {
    // Reference-validated: inner port reads 4 bits, outer implicit net is 1.
    let src = r#"
module sink(input logic [3:0] vld);
  logic [31:0] w_in;
  assign w_in = $bits(vld);
endmodule
module tb;
  sink u (.vld(nv_vld));
  logic [31:0] w_out;
  assign w_out = $bits(nv_vld);
  initial #1;
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    let g = |n: &str| {
        sim.get_signal(n)
            .or_else(|| sim.get_signal(&format!("tb.{}", n)))
            .unwrap_or_else(|| panic!("signal not found: {}", n))
            .to_u64()
            .unwrap()
    };
    assert_eq!(g("u.w_in"), 4, "port side keeps its declared width");
    assert_eq!(
        g("w_out"),
        1,
        "implicit net is scalar, matching the reference"
    );
}
