//! Sibling-repo (xezim-core / xezim-parser) audit — three defects, all
//! reference-validated. The rest of the sweep (x/z propagation, literals,
//! macros, shifts at width boundaries, generate case/nesting, real
//! conversions, precedence corners) matched the reference untouched.
//!
//! 1. **§11.6.1 WIDE signed division/modulo.** The >64-bit arm of
//!    `Value::div`/`modulo` went straight to u128 arithmetic with no
//!    signedness check, so a 128-bit `-5 / 3` divided the raw
//!    two's-complement pattern (a huge positive quotient) and `-5 % 3` gave
//!    +2. The 64-bit arm had the signed path all along.
//! 2. **Wide `**`.** `power` accumulated in u64 regardless of operand width,
//!    so `128'd2 ** 100` wrapped to 0. Accumulates in u128 now, with an
//!    early exit once the accumulator hits 0.
//! 3. **§11.3.2 `**` associativity.** The parser bound `**` right-to-left
//!    (like most general-purpose languages); SystemVerilog makes ALL binary
//!    operators left-associative — only `?:` is right-associative. So
//!    `2 ** 3 ** 2` is `(2**3)**2 = 64`, not `2**(3**2) = 512`.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

fn hex(sim: &xezim::compiler::Simulator, n: &str) -> String {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("missing {n}"))
        .to_hex_string()
}

/// §11.6.1 in the wide lane: quotient and remainder keep their signs.
#[test]
fn wide_signed_division_and_modulo() {
    let src = r#"
module top;
  logic signed [127:0] sa, sb, q, m;
  logic [127:0] uq;
  int q32, m32;
  initial begin
    sa = -128'sd5; sb = 128'sd3;
    q = sa / sb;  m = sa % sb;
    q32 = q; m32 = m;
    // the unsigned wide path is untouched
    uq = 128'hFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFB / 128'd3;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "q32") as u32 as i32,
        -1,
        "-5 / 3 truncates toward zero"
    );
    assert_eq!(
        u(&sim, "m32") as u32 as i32,
        -2,
        "-5 % 3 keeps the dividend's sign"
    );
    assert_eq!(
        hex(&sim, "uq"),
        "55555555555555555555555555555553",
        "unsigned wide division unchanged"
    );
}

/// A wide power survives past 64 bits.
#[test]
fn wide_power_accumulates_in_128_bits() {
    let src = r#"
module top;
  logic [127:0] p100, p64, p_even;
  logic [63:0] p_narrow;
  initial begin
    p100 = 128'd2 ** 100;
    p64  = 128'd2 ** 64;
    p_even = 128'd4 ** 70;        // saturates the width -> wraps to 0
    p_narrow = 64'd3 ** 5;        // the narrow path is unchanged
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        hex(&sim, "p100"),
        "00000010000000000000000000000000",
        "2**100"
    );
    assert_eq!(
        hex(&sim, "p64"),
        "00000000000000010000000000000000",
        "2**64"
    );
    assert_eq!(
        hex(&sim, "p_even"),
        "00000000000000000000000000000000",
        "even base past the width"
    );
    assert_eq!(u(&sim, "p_narrow"), 243);
}

/// §11.3.2: `**` associates LEFT to right.
#[test]
fn power_is_left_associative() {
    let src = r#"
module top;
  int chain, parens_l, parens_r, with_mul;
  initial begin
    chain    = 2 ** 3 ** 2;
    parens_l = (2 ** 3) ** 2;
    parens_r = 2 ** (3 ** 2);
    with_mul = 2 * 3 ** 2;     // ** binds tighter than *
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "chain"), 64, "left-associative chain");
    assert_eq!(u(&sim, "parens_l"), 64);
    assert_eq!(u(&sim, "parens_r"), 512, "explicit parens still override");
    assert_eq!(u(&sim, "with_mul"), 18, "precedence over * unchanged");
}

/// ===/!== widening of a signed integer LITERAL sign-extends against a wider
/// unsigned operand (commercial consensus; ivtest `sv_cast_packed_array`):
/// `64'hFF..F0 !== -16` is FALSE. A signed VARIABLE in the same position
/// zero-extends per the propagated unsigned type — the reference draws the
/// line at literal-vs-variable, and so does xezim (at the eval site, since
/// Value has no source shape).
#[test]
fn case_equality_extends_by_own_sign() {
    let src = r#"
module top;
  logic [63:0] u64;
  logic signed [7:0] s8;
  int ne, eq, sx_eq, sx_ne;
  initial begin
    u64 = 64'hFFFF_FFFF_FFFF_FFF0;
    ne = (u64 !== -16);
    eq = (u64 === -16);
    s8 = -1;
    sx_eq = (16'hFFFF === s8);
    sx_ne = (16'h00FF === s8);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "ne"),
        0,
        "the LITERAL -16 sign-extends against the unsigned 64-bit"
    );
    assert_eq!(u(&sim, "eq"), 1);
    assert_eq!(
        u(&sim, "sx_eq"),
        0,
        "a signed VARIABLE zero-extends (propagated unsigned)"
    );
    assert_eq!(u(&sim, "sx_ne"), 1);
}
