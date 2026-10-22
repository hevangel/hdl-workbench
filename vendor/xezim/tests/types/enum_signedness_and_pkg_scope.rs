//! ivtest FAIL_OUT mining, round 1 — four defects, all reference-validated.
//!
//! 1. §6.19 **enum signedness**: an enum takes its BASE type's signedness, and
//!    the default base is `int` (signed). `is_type_signed` had no `Enum` arm,
//!    so every enum was unsigned: `enum shortint {A=-1,...} es;` read `es` as
//!    65534 and `es.first()` as 65535, and `es.first() < 0` was false.
//!    (ivtest enum_method_signed1..4)
//! 2. §6.19.6 **next/prev matched raw u64s**, so a value held at a different
//!    width than the member table never matched and every call fell through to
//!    the invalid-value default. Now compared at the enum's width, with the
//!    results carrying the enum's signedness.
//! 3. §20.5 **`$is_signed` was not implemented at all** — it silently answered
//!    0, which made `!$is_signed(x)` pass by accident and `$is_signed(x)` fail
//!    for genuinely signed operands. (ivtest struct_signed,
//!    struct_member_signed. Note the reference simulator also answers 0 here;
//!    the LRM and ivtest agree that a `struct packed signed` is signed.)
//! 4. §6.16.10 **`realtoa` used `%f`** ("11.100000") where the decimal
//!    representation is `%g` ("11.1", "1e+20"). (part of ivtest sv_string6)
//! 5. §26.3 **package parameters were registered only under their BARE name**,
//!    so two packages declaring the same name collided and `p1::step` read
//!    whichever package elaborated last. (part of ivtest sv_package2)

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn enum_takes_base_type_signedness() {
    let src = r#"
module tb;
  enum shortint  { A = -1, B = -2, C = -3 } es;      // signed base
  enum bit [15:0]{ X = 65535, Y = 65534 } eu;        // unsigned base
  enum           { D = 32'hFFFFFFFF, E = 1 } ed;     // default base int -> signed
  int sv, uv, dv, first_neg, cmp;
  initial begin
    es = B; eu = Y; ed = D;
    sv = es;              // -2, not 65534
    uv = eu;              // 65534 (unsigned base)
    dv = ed;              // -1, not 4294967295
    first_neg = (es.first() < 0);
    cmp = ($signed(eu.first()) < 0);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "sv") as i32, -2, "signed base type");
    assert_eq!(u(&sim, "uv"), 65534, "unsigned base type unchanged");
    assert_eq!(
        u(&sim, "dv") as i32,
        -1,
        "default enum base is int (signed)"
    );
    assert_eq!(u(&sim, "first_neg"), 1, "es.first() must be negative");
    assert_eq!(u(&sim, "cmp"), 1, "$signed() of an unsigned-base member");
}

#[test]
fn enum_next_prev_match_at_enum_width() {
    let src = r#"
module tb;
  enum shortint { A = -1, B = -2, C = -3 } es;
  int nx, pv, fst, lst;
  initial begin
    es = B;
    nx = es.next();   // C = -3
    pv = es.prev();   // A = -1
    fst = es.first(); // -1
    lst = es.last();  // -3
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "nx") as i32, -3, "next() must find the member");
    assert_eq!(u(&sim, "pv") as i32, -1, "prev() must find the member");
    assert_eq!(u(&sim, "fst") as i32, -1);
    assert_eq!(u(&sim, "lst") as i32, -3);
}

#[test]
fn is_signed_reports_declared_signedness() {
    let src = r#"
module tb;
  struct packed          { logic [15:0] x; } s1;
  struct packed unsigned { logic [15:0] x; } s2;
  struct packed signed   { logic [15:0] x; } s3;
  struct packed { logic signed [15:0] y; logic [7:0] z; } sm;
  int i1, i2, i3, iy, iz, ii, iu;
  int  si;
  int unsigned ui;
  initial begin
    i1 = $is_signed(s1); i2 = $is_signed(s2); i3 = $is_signed(s3);
    iy = $is_signed(sm.y); iz = $is_signed(sm.z);
    ii = $is_signed(si);  iu = $is_signed(ui);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "i1"), 0, "unsigned by default");
    assert_eq!(u(&sim, "i2"), 0, "explicitly unsigned");
    assert_eq!(u(&sim, "i3"), 1, "struct packed signed");
    assert_eq!(u(&sim, "iy"), 1, "signed member");
    assert_eq!(u(&sim, "iz"), 0, "unsigned member");
    assert_eq!(u(&sim, "ii"), 1, "int is signed");
    assert_eq!(u(&sim, "iu"), 0, "int unsigned is not");
}

#[test]
fn realtoa_uses_decimal_g_format() {
    let src = r#"
module tb;
  string s;
  int l1, l2, l3, ok1, ok2, ok3;
  initial begin
    s.realtoa(11.1);  ok1 = (s == "11.1");  l1 = s.len();
    s.realtoa(100.0); ok2 = (s == "100");   l2 = s.len();
    s.realtoa(1e20);  ok3 = (s == "1e+20"); l3 = s.len();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "ok1"), 1, "11.1 not 11.100000");
    assert_eq!(u(&sim, "ok2"), 1, "trailing zeros stripped");
    assert_eq!(u(&sim, "ok3"), 1, "exponent form for large magnitudes");
    assert_eq!(u(&sim, "l1"), 4);
    assert_eq!(u(&sim, "l2"), 3);
    assert_eq!(u(&sim, "l3"), 5);
}

#[test]
fn same_named_package_params_do_not_collide() {
    let src = r#"
package p1;
  localparam step = 1;
  localparam only1 = 11;
endpackage
package p2;
  localparam step = 2;
  localparam only2 = 22;
endpackage
module tb;
  int a, b, c, d;
  initial begin
    a = p1::step;   // must be p1's, not whichever elaborated last
    b = p2::step;
    c = p1::only1;
    d = p2::only2;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 1, "p1::step must resolve to p1's value");
    assert_eq!(u(&sim, "b"), 2, "p2::step");
    assert_eq!(u(&sim, "c"), 11);
    assert_eq!(u(&sim, "d"), 22);
}
