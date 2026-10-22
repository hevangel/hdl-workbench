//! §11.6.1 — `~` and unary `-` are CONTEXT-determined: the operand is
//! extended to the assignment's width BEFORE the operation, not after.
//! Reference-validated.
//!
//! The compiler already threaded the context width into the operand, but a
//! plain signal load returns its DECLARED width, so the extension never
//! happened: `logic [31:0] r = ~a;` with an 8-bit `a` computed `~a` in 8 bits
//! and zero-extended the result, yielding 0000004b where ffffff4b is
//! required. Same for `-a` (0000004c instead of ffffff4c).
//!
//! Binary operators were already correct (`a - 8'hFF` gives ffffffb5), which
//! is what made this look like a signedness quirk rather than a width one.
//!
//! The reduction operators and `!` are SELF-determined and must NOT be
//! resized — a widened operand would change `&a` — so they are excluded and
//! covered here as a guard.

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
  logic [7:0]        a  = 8'hB4;    // 180
  logic signed [7:0] sn = -8'sd12;  // 0xF4
  logic [7:0]        ones = 8'hFF;

  logic [31:0] r_not, r_neg, r_sub, r_not_sgn, r_neg_sgn;
  logic [7:0]  r_not_same, r_neg_same;
  logic        r_redand, r_redor, r_lognot;
  logic [31:0] r_redand_w;

  assign r_not      = ~a;        // ctx 32 -> ffffff4b
  assign r_neg      = -a;        // ctx 32 -> ffffff4c
  assign r_sub      = a - 8'hFF; // binary, already correct -> ffffffb5
  assign r_not_sgn  = ~sn;       // signed operand sign-extends first -> 0000000b
  assign r_neg_sgn  = -sn;       // -(-12) -> 0000000c
  assign r_not_same = ~a;        // ctx 8 -> 4b (unchanged)
  assign r_neg_same = -a;        // ctx 8 -> 4c (unchanged)

  // Self-determined: must NOT be widened by the context.
  assign r_redand   = &ones;     // 1
  assign r_redor    = |a;        // 1
  assign r_lognot   = !a;        // 0
  assign r_redand_w = &ones;     // still 1, not 0 from a widened operand

  initial #1;
endmodule
"#;

#[test]
fn unary_not_and_minus_extend_to_the_context_width() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_not"), 0xFFFF_FF4B, "~a in a 32-bit context");
    assert_eq!(u(&sim, "r_neg"), 0xFFFF_FF4C, "-a in a 32-bit context");
    assert_eq!(
        u(&sim, "r_sub"),
        0xFFFF_FFB5,
        "binary - was already correct"
    );
    assert_eq!(
        u(&sim, "r_not_sgn"),
        0x0000_000B,
        "~sn sign-extends the operand first"
    );
    assert_eq!(u(&sim, "r_neg_sgn"), 0x0000_000C, "-sn");
}

#[test]
fn unary_in_a_same_width_context_is_unchanged() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_not_same"), 0x4B, "~a in an 8-bit context");
    assert_eq!(u(&sim, "r_neg_same"), 0x4C, "-a in an 8-bit context");
}

#[test]
fn self_determined_unaries_are_not_widened() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_redand"), 1, "&8'hFF");
    assert_eq!(u(&sim, "r_redor"), 1, "|a");
    assert_eq!(u(&sim, "r_lognot"), 0, "!a");
    assert_eq!(
        u(&sim, "r_redand_w"),
        1,
        "&8'hFF in a 32-bit context — widening the operand would make this 0"
    );
}
