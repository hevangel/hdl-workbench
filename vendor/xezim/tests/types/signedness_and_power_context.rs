//! §11.6.1 / §11.8.1 — context width into `**`, and mixed-sign widening.
//! Reference-validated, both evaluation paths.
//!
//! Three defects:
//!  * `a ** 2` with a non-constant base BAILED the whole block to the
//!    interpreter (const-fold only), and the interpreter computed the power
//!    at the operand's own width — `a ** 2` in a 32-bit context read 0x90
//!    for 0x7e90. A real Pow insn now exists; the LEFT operand widens to the
//!    operation width first (the exponent stays self-determined).
//!  * `unsigned'(sa)` compiled to a NO-OP: the operand kept its runtime
//!    signed flag and the context Resize SIGN-extended (fffffff4 for
//!    000000f4). ClearSigned insn added.
//!  * mixed-sign `sa + b` — the expression is UNSIGNED if any operand is
//!    unsigned, so widening must zero-extend BOTH operands; the runtime
//!    Resize extended by each value's own flag instead (fffffff9 for
//!    000000f9). Static expression-signedness now drives a ClearSigned
//!    before the widen; the all-signed case still sign-extends.
//!
//! The $display path was already correct for the sign cases — which is why
//! this hid: displays agreed with the reference while every assign was wrong.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

const SRC: &str = r#"
module tb;
  logic        [7:0] a  = 8'hB4;    // 180
  logic signed [7:0] sa = -8'sd12;  // 0xF4
  logic        [7:0] b  = 8'h05;
  logic [31:0] r_pow, r_mix, r_uns, r_sgn;
  logic [15:0] r_pow16;
  logic [7:0]  r_pow8;
  assign r_pow   = a ** 2;
  assign r_pow16 = a ** 2;
  assign r_pow8  = a ** 2;          // self-width: truncation IS correct here
  assign r_mix   = sa + b;
  assign r_uns   = unsigned'(sa);
  assign r_sgn   = sa + 8'sh01;     // all-signed: must still sign-extend
  initial #1;
endmodule
"#;

#[test]
fn power_left_operand_takes_the_context_width() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_pow"), 0x7E90, "180**2 in a 32-bit context");
    assert_eq!(u(&sim, "r_pow16"), 0x7E90, "and in a 16-bit context");
    assert_eq!(
        u(&sim, "r_pow8"),
        0x90,
        "8-bit context truncates, correctly"
    );
}

#[test]
fn mixed_sign_and_unsigned_cast_zero_extend() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "r_mix"),
        0x0000_00F9,
        "sa + b is UNSIGNED (§11.8.1)"
    );
    assert_eq!(u(&sim, "r_uns"), 0x0000_00F4, "unsigned'(sa) zero-extends");
    assert_eq!(
        u(&sim, "r_sgn"),
        0xFFFF_FFF5,
        "all-signed still sign-extends"
    );
}
