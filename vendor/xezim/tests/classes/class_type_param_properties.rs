//! §8.25 — a class property declared with a TYPE PARAMETER has the type that
//! parameter is BOUND to on the object.
//!
//! Such a property used to carry no type at all at run time: it simply held
//! whatever `Value` was last stored. So `$bits` reported the stored value's
//! width rather than the declared one, an over-wide store was never truncated
//! (`T = byte` happily kept 32 bits), and a struct-typed parameter had no
//! field layout — `st.addr` read x even though the raw storage was correct.
//!
//! The property's declared name is the PARAMETER, so it is resolved through
//! the bindings the object was constructed with. Three separate gaps had to
//! close for that to work end to end:
//!
//!   * a type parameter's default was only recorded when it named a user type,
//!     so `#(type T = int)` had no default to bind;
//!   * a package-scoped type argument (`pk::pkt_t`) parses as a MemberAccess
//!     rather than an Ident, so it was classified as a VALUE argument, matched
//!     against the value parameters, and dropped;
//!   * a named connection carries its argument as a TypeLiteral (`.T(int)`),
//!     which was likewise not recognised as a type.
//!
//! Deliberately still unclamped: `bit`/`logic`/`reg`. The parser drops the
//! packed range of a type ARGUMENT, so `#(logic [63:0])` and `#(logic)` are
//! indistinguishable here and clamping to 1 would destroy data.

use xezim::simulate;

/// Width and truncation follow the BOUND type, not the last value stored.
const CLAMP: &str = r#"
module tb;
  class box #(parameter type T = int);
    T p;
  endclass
  int byte_after, byte_over_bits, byte_over_val;
  int int_default_bits, vec_bits, vec_val;
  initial begin
    box #(byte)         c;
    box #(logic [63:0]) v;
    box                 d;
    c = new(); v = new(); d = new();
    c.p = 8'h5A;
    byte_after = $bits(c.p);
    c.p = 32'h12345678;          // over-wide store into a `byte`
    byte_over_bits = $bits(c.p);
    byte_over_val  = c.p;
    d.p = 8'h5A;                 // T defaults to int
    int_default_bits = $bits(d.p);
    v.p = 64'hFEDCBA98;          // range lost at parse: must stay unclamped
    vec_bits = $bits(v.p);
    vec_val  = v.p;
  end
endmodule
"#;

/// A struct-typed parameter gives the property a field layout, from a `$unit`
/// typedef, a package-scoped one, and the declared default alike.
const STRUCT_PARAM: &str = r#"
package pk;
  typedef struct packed { logic [7:0] addr; logic [31:0] data; } pkt_t;
endpackage
typedef struct packed { logic [7:0] a; logic [31:0] d; } loc_t;
module tb;
  class box #(parameter type ST = loc_t);
    ST s;
  endclass
  int dflt_a, dflt_d, expl_a, expl_d, pkg_a, pkg_d, pkg_bits;
  initial begin
    box              b_dflt;
    box #(loc_t)     b_expl;
    box #(pk::pkt_t) b_pkg;
    b_dflt = new(); b_expl = new(); b_pkg = new();
    b_dflt.s = 40'h7F12345678;
    b_expl.s = 40'h7F12345678;
    b_pkg.s  = 40'h7F12345678;
    dflt_a = b_dflt.s.a;    dflt_d = b_dflt.s.d;
    expl_a = b_expl.s.a;    expl_d = b_expl.s.d;
    pkg_a  = b_pkg.s.addr;  pkg_d  = b_pkg.s.data;
    pkg_bits = $bits(b_pkg.s);
  end
endmodule
"#;

/// A `string`-bound parameter must NOT be bit-clamped — truncating would drop
/// leading characters. This is the case the original code protected by never
/// clamping anything.
const STRING_PARAM: &str = r#"
module tb;
  class box #(parameter type T = string);
    T v;
  endclass
  string got_default, got_explicit;
  initial begin
    box            bd;
    box #(string)  be;
    bd = new(); be = new();
    bd.v = "a_reasonably_long_string";
    be.v = "a_reasonably_long_string";
    got_default  = bd.v;
    got_explicit = be.v;
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

fn s(sim: &xezim::compiler::Simulator, n: &str) -> String {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_sv_string()
}

#[test]
fn type_param_property_clamps_to_its_bound_type() {
    let sim = simulate(CLAMP, 100).expect("simulate failed");
    assert_eq!(u(&sim, "byte_after"), 8, "T = byte gives an 8-bit property");
    assert_eq!(
        u(&sim, "byte_over_bits"),
        8,
        "an over-wide store stays 8 bits"
    );
    assert_eq!(
        u(&sim, "byte_over_val"),
        0x78,
        "32'h12345678 into a byte keeps 0x78"
    );
    assert_eq!(
        u(&sim, "int_default_bits"),
        32,
        "the `int` DEFAULT binds and sizes"
    );
}

/// The parser drops a type argument's packed range, so these must keep the
/// old unclamped behavior rather than collapse to 1 bit.
#[test]
fn vector_type_argument_stays_unclamped() {
    let sim = simulate(CLAMP, 100).expect("simulate failed");
    // Unclamped means the property keeps the stored value as-is, so `$bits`
    // reports the assigned literal's width (64) rather than a declared one.
    // The assertion that matters is that it is not 1.
    assert_eq!(
        u(&sim, "vec_bits"),
        64,
        "unclamped: reports the stored 64'h literal's width"
    );
    assert_eq!(
        u(&sim, "vec_val"),
        0xFEDC_BA98,
        "the value must not be truncated to 1 bit"
    );
}

#[test]
fn struct_type_param_property_projects_fields() {
    let sim = simulate(STRUCT_PARAM, 100).expect("simulate failed");
    assert_eq!(u(&sim, "dflt_a"), 0x7F, "declared default struct type");
    assert_eq!(
        u(&sim, "dflt_d"),
        0x1234_5678,
        "declared default struct type"
    );
    assert_eq!(u(&sim, "expl_a"), 0x7F, "explicit $unit typedef");
    assert_eq!(u(&sim, "expl_d"), 0x1234_5678, "explicit $unit typedef");
    assert_eq!(u(&sim, "pkg_a"), 0x7F, "package-scoped type argument");
    assert_eq!(
        u(&sim, "pkg_d"),
        0x1234_5678,
        "package-scoped type argument"
    );
    assert_eq!(u(&sim, "pkg_bits"), 40, "packed struct width");
}

/// Guards the exception the original code existed to protect.
#[test]
fn string_type_param_property_is_not_truncated() {
    let sim = simulate(STRING_PARAM, 100).expect("simulate failed");
    assert_eq!(
        s(&sim, "got_default"),
        "a_reasonably_long_string",
        "defaulted string T"
    );
    assert_eq!(
        s(&sim, "got_explicit"),
        "a_reasonably_long_string",
        "explicit string T"
    );
}
