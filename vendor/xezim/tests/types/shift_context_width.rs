//! §11.4.10 — a shift's LEFT operand is context-determined.
//!
//! It is resized to the expression's width BEFORE the shift happens. xezim
//! already did this for `<<`, but `>>` and `>>>` shifted at the operand's own
//! narrow width and only then widened, so the bits vacated by a logical right
//! shift came from the narrow result rather than from the sign extension:
//!
//! ```systemverilog
//! logic signed [3:0] s = -4;     // 4'b1100
//! int i = s >> 1;                // 6  — wrong; must be 32'h7FFF_FFFE
//! ```
//!
//! Only `>>` exposes it. `>>>` preserves the sign either way and `<<` was
//! already handled, so every other spelling agreed — which is why a narrow
//! signed value silently lost its high bits only in this one form.
//!
//! Verified byte-identical to a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The context width drives the extension: the same expression gives different
/// (correct) answers in a 32-bit, 8-bit and 4-bit target.
#[test]
fn right_shift_widens_its_left_operand_to_the_context() {
    let src = r#"
module top;
  logic signed [3:0] s = -4;      // 4'b1100
  int         i32;
  logic [7:0] r8;
  logic [3:0] r4;
  initial begin
    i32 = s >> 1;
    r8  = s >> 1;
    r4  = s >> 1;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "i32") as u32,
        0x7FFF_FFFE,
        "32-bit context sign-extends first"
    );
    assert_eq!(u(&sim, "r8") & 0xFF, 0x7E, "8-bit context");
    assert_eq!(u(&sim, "r4") & 0xF, 0x6, "4-bit context needs no extension");
}

/// The guards: an UNSIGNED operand zero-extends (unchanged), `>>>` preserves
/// the sign, `<<` is unchanged, and a self-determined context is unaffected.
#[test]
fn other_shift_forms_are_unchanged() {
    let src = r#"
module top;
  logic signed [3:0] s = -4;
  logic [3:0]        un = 4'b1100;
  int a, b, c, d;
  initial begin
    a = un >> 1;     // unsigned: 6
    b = s >>> 1;     // arithmetic: -2
    c = s << 1;      // -8
    d = 4'sb1100 >> 1;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 6, "unsigned left operand zero-extends");
    assert_eq!(
        u(&sim, "b") as u32 as i32,
        -2,
        "arithmetic shift keeps the sign"
    );
    assert_eq!(u(&sim, "c") as u32 as i32, -8, "left shift unchanged");
    assert_eq!(
        u(&sim, "d") as u32,
        0x7FFF_FFFE,
        "a signed literal behaves the same"
    );
}
