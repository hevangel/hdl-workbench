//! §6.11.1 / §5.7.1 (ivtest br_gh243): storage keeps the ELEMENT'S declared
//! signedness, never the rvalue's. An unsized decimal literal is signed, so
//! `barr[i] = 9` stamped is_signed onto a `bit [3:0]` cell and every later
//! read sign-extended it: `array[i] != i` failed for i >= 8, and
//! `$display("%0d", barr[i])` printed -7. Enforced as an invariant in the
//! write_sig! macro: table slots always carry `signal_signed[id]`.
//! Reference-validated (1-D, 2-D, signed element types, comparisons).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
        & 0xFFFF_FFFF
}

/// Unsigned elements written with (signed) unsized literals stay unsigned.
#[test]
fn unsigned_elements_do_not_inherit_literal_signedness() {
    let src = r#"
module tb;
  bit   [3:0] barr[15:0];
  logic [3:0] larr[1:0];
  bit   [3:0] a2[1:0][1:0];
  integer i;
  int mismatches, l_ok, a2_ok, disp;
  initial begin
    for (i = 0; i < 16; i++) barr[i] = i;
    mismatches = 0;
    for (i = 0; i < 16; i++) if (barr[i] != i) mismatches++;
    larr[0] = 9;
    l_ok = (larr[0] == 9);
    a2[1][0] = 9;
    a2_ok = (a2[1][0] == 9);
    disp = barr[9];              // must widen to 9, not -7
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "mismatches"),
        0,
        "br_gh243: MSB-set elements compared signed"
    );
    assert_eq!(u(&sim, "l_ok"), 1, "4-state unsigned element");
    assert_eq!(u(&sim, "a2_ok"), 1, "2-D element");
    assert_eq!(
        u(&sim, "disp"),
        9,
        "unsigned element widens with zero-extension"
    );
}

/// Declared-signed element types still read back signed.
#[test]
fn signed_element_types_stay_signed() {
    let src = r#"
module tb;
  byte sarr[1:0];
  int  iarr[1:0];
  int s_ok, i_ok, neg;
  initial begin
    sarr[0] = -7;
    iarr[0] = -7;
    s_ok = (sarr[0] == -7);
    i_ok = (iarr[0] == -7);
    neg = (sarr[0] < 0);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "s_ok"), 1, "byte element sign-extends");
    assert_eq!(u(&sim, "i_ok"), 1, "int element keeps value");
    assert_eq!(u(&sim, "neg"), 1, "byte element compares signed");
}
