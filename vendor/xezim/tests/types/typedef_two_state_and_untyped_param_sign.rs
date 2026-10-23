//! Two high-severity findings from the IEEE 1800 audit of xezim-core, both
//! reference-validated. Neither is covered by any existing test corpus —
//! they were silent wrong answers.
//!
//! 1. **§6.8 / Table 6-7 + §6.18 — a 2-state variable reached through a
//!    TYPEDEF initialized to x instead of 0.** `is_type_two_state` has no
//!    `TypeReference` arm, so the alias hid the 2-state-ness. The identical
//!    DIRECT declaration was always correct, which is what hid it. A typedef
//!    is a pure alias (§6.18) and cannot change the initial value.
//!
//! 2. **§6.20.2 + §5.7.1 — an untyped parameter took its width from a sized
//!    literal but was always marked SIGNED.** `parameter P = 8'hF0;` read
//!    -16 instead of 240, so `P > 100` was false. A sized literal is unsigned
//!    unless it carries the `s` designator; an unsized decimal stays signed.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
        & 0xFFFF_FFFF
}

/// A typedef must not change the type's initial value, at any scope.
#[test]
fn two_state_default_survives_a_typedef() {
    let src = r#"
package pk;
  typedef bit [7:0] pbt8;
endpackage
typedef bit [7:0] bt8;
typedef int       ti;
module tb;
  bt8       via_td;      // through a $unit typedef
  ti        via_td_int;
  pk::pbt8  via_pkg;     // through a package typedef
  bt8       arr[0:1];    // unpacked array of the typedef
  bit [7:0] direct;      // direct form (was already correct)
  int       direct_int;
  logic [7:0] four_state;   // 4-state MUST still be x
  int x_td, x_tdi, x_pkg, x_arr, x_dir, x_diri, x_4s, val_td;
  initial begin
    x_td   = $isunknown(via_td);
    x_tdi  = $isunknown(via_td_int);
    x_pkg  = $isunknown(via_pkg);
    x_arr  = $isunknown(arr[0]);
    x_dir  = $isunknown(direct);
    x_diri = $isunknown(direct_int);
    x_4s   = $isunknown(four_state);
    val_td = via_td;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "x_td"), 0, "typedef'd bit vector must start at 0");
    assert_eq!(u(&sim, "x_tdi"), 0, "typedef'd int");
    assert_eq!(u(&sim, "x_pkg"), 0, "package typedef");
    assert_eq!(u(&sim, "x_arr"), 0, "unpacked array of a typedef");
    assert_eq!(u(&sim, "x_dir"), 0, "direct form unchanged");
    assert_eq!(u(&sim, "x_diri"), 0);
    assert_eq!(u(&sim, "x_4s"), 1, "4-state must STILL initialize to x");
    assert_eq!(u(&sim, "val_td"), 0);
}

/// §5.7.1: sized literal unsigned unless `s`; unsized decimal stays signed.
#[test]
fn untyped_parameter_takes_its_value_signedness() {
    let src = r#"
module tb;
  parameter  MP = 8'hF0;      // unsigned -> 240
  localparam ML = 6'b110000;  // unsigned -> 48
  localparam P7 = 3'b101;     // unsigned -> 5
  localparam SS = 8'shF0;     // explicitly signed -> -16
  localparam UN = 240;        // unsized decimal -> signed int, 240
  localparam NEG = -5;        // unsized negative stays signed
  int mp, ml, p7, ss, un, neg, cmp, wid;
  initial begin
    mp = MP; ml = ML; p7 = P7; ss = SS; un = UN; neg = NEG;
    cmp = (MP > 100);
    wid = $bits(MP);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "mp"), 240, "sized hex literal is unsigned");
    assert_eq!(u(&sim, "ml"), 48);
    assert_eq!(u(&sim, "p7"), 5);
    assert_eq!(u(&sim, "ss") as i32, -16, "explicit 's' stays signed");
    assert_eq!(u(&sim, "un"), 240, "unsized decimal");
    assert_eq!(u(&sim, "neg") as i32, -5, "unsized negative stays signed");
    assert_eq!(u(&sim, "cmp"), 1, "MP > 100 must be true");
    assert_eq!(u(&sim, "wid"), 8, "width still comes from the literal");
}

/// §6.19.1 (grammar A.2.2.1): `enum_base_type` may be a `type_identifier`, and
/// there is NO enum name between `enum` and `{`. The parser was discarding
/// that identifier as a supposed enum name, so the enum silently fell back to
/// the 32-bit `int` default: `$bits` read 32 instead of the base's width, and
/// a signed base lost its sign.
#[test]
fn enum_base_type_may_be_a_typedef() {
    let src = r#"
module tb;
  typedef logic [3:0] nib_t;
  typedef byte        sb_t;
  typedef enum nib_t { A0, A1 } en_t;
  typedef enum sb_t  { B0 = -2, B1 } eb_t;
  typedef enum logic [3:0] { C0, C1 } inline_t;   // control: inline base
  eb_t e2;
  en_t e1;
  int b_en, b_eb, b_in, v_b0, lt0, v_a1;
  initial begin
    b_en = $bits(en_t);
    b_eb = $bits(eb_t);
    b_in = $bits(inline_t);
    e2 = B0; v_b0 = e2; lt0 = (e2 < 0);
    e1 = A1; v_a1 = e1;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "b_en"), 4, "base type nib_t is 4 bits, not int");
    assert_eq!(u(&sim, "b_eb"), 8, "base type sb_t (byte) is 8 bits");
    assert_eq!(u(&sim, "b_in"), 4, "inline base still works");
    assert_eq!(u(&sim, "v_b0") as i32, -2, "signed base keeps its sign");
    assert_eq!(u(&sim, "lt0"), 1, "e2 < 0 must be true");
    assert_eq!(u(&sim, "v_a1"), 1);
}
