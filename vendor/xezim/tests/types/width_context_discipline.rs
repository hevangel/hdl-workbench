//! §11.6.1/§11.8.1/§11.4.10/§11.4.11 — operand width/sign discipline for
//! shifts, div/mod, ternary, unary, and relational operators. Reference-
//! validated three ways per expression (reference, $display, assign) by a
//! differential audit; every value here is the reference simulator's.
//!
//! The umbrella defect: THREE different width sources disagreed —
//!  * `expr_max_width` (carry-aware, deliberately over-reporting) leaked into
//!    LRM context positions: a continuous assign's compile context and the
//!    interpreter's shift-operand width both used it, so `(a<<4)>>2` on an
//!    8-bit net computed the inner shift at 12 bits and the dropped carry
//!    returned (0x8c for 0x0c) — while the IDENTICAL always_comb was correct;
//!  * right-shift/div/mod operands never got their context Resize on the VM
//!    path (signed 8-bit >> in a 32-bit context zero-extended after shifting;
//!    -128/-1 divided at 8 bits; x-results spanned only the operand width);
//!  * ternary arms and interpreter unary/relational operands took the wrong
//!    context entirely (mixed-sign arms sign-extended; `~a + 32'd0` printed
//!    0000005c for ffffff5c; an assignment's width leaked into `>` operands).

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
  logic [7:0] a = 8'ha3, b = 8'h1c, zed = 8'h00;
  logic signed [7:0] sa = 8'h9c, smin = 8'sh80, sm1 = -8'sd1;
  logic [0:0] c1 = 1'b1;
  logic [7:0]  r1, r2, r3, r4;
  logic [31:0] w1, w3, w4, u1, u2, t32;
  logic        xz1;
  assign r1 = (a << 4) >> 2;
  assign r2 = (a << 4) >>> 2;
  assign r3 = (a + a) >> 1;
  assign r4 = (b - a) >> 1;
  assign w1 = smin / sm1;
  assign w3 = sa >> 3;
  assign w4 = c1 ? sa : b;
  assign u1 = ~a + 32'd0;
  assign u2 = -a + 32'd0;
  assign xz1 = (^(a / zed)) === 1'bx;   // div-by-zero -> x across the width
  initial begin #1;
    t32 = (a + a) > 8'h50;              // relational operands stay 8-bit
  end
endmodule
"#;

#[test]
fn shift_operands_take_the_lrm_context_not_the_carry_width() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r1"), 0x0C, "(a<<4)>>2 at 8 bits — was 0x8c");
    assert_eq!(u(&sim, "r2"), 0x0C, ">>> variant");
    assert_eq!(u(&sim, "r3"), 0x23, "(a+a)>>1 — carry must not return");
    assert_eq!(u(&sim, "r4"), 0x3C, "(b-a)>>1 — borrow must not return");
    assert_eq!(
        u(&sim, "w3"),
        0x1FFF_FFF3,
        "signed >> in a 32-bit context sign-extends FIRST"
    );
}

#[test]
fn div_mod_are_context_determined_in_both_operands() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "w1"),
        0x0000_0080,
        "-128/-1 at 32 bits is +128, not -128"
    );
    assert_eq!(u(&sim, "xz1"), 1, "a/0 is x across the full context width");
}

#[test]
fn ternary_unary_and_relational_context() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w4"), 0x0000_009C, "mixed-sign ternary is unsigned");
    assert_eq!(
        u(&sim, "u1"),
        0xFFFF_FF5C,
        "~a extends before the op (both paths)"
    );
    assert_eq!(u(&sim, "u2"), 0xFFFF_FF5D, "-a likewise");
    assert_eq!(
        u(&sim, "t32"),
        0,
        "assignment width must NOT leak into > operands"
    );
}
