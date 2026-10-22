//! §8.25 / §6.9.1 — sizing a CLASS PROPERTY whose packed range is a constant
//! expression rather than a literal.
//!
//! `elaborate_class` sized every property with NO parameters in scope
//! (`resolve_type_width(dt, None, ..)`). `const_eval_i64_with_params` then
//! returned `None` for the range bounds, `resolve_type_width` silently skipped
//! the unresolvable dimension, and the property came out **1 bit** — so
//! `$bits(obj.p)` reported 1 and `fit_class_prop` truncated every store to one
//! bit. This held for a class parameter (`bit [W-1:0]`) and equally for any
//! module, package or `$unit` parameter used in a class property's range.
//!
//! Widths are now resolved against the enclosing scope's parameters overlaid
//! with the class's own parameters and body localparams. Because one
//! `ElaboratedClass` is shared by every specialization of a base class, that
//! width is the DEFAULT-parameter one; a property whose range depends on a
//! class parameter is re-resolved per instance from the parameter bindings
//! the instance carries (see `Simulator::respec_packed_width`).

use xezim::simulate;

/// Every parameter scope that can appear in a class property's packed range.
const SCOPES: &str = r#"
package pk;
  parameter int PW = 20;
endpackage
parameter int UW = 24;
module tb;
  import pk::*;
  parameter int MW = 12;
  localparam int LW = 10;
  class c1;
    localparam int CW = 6;
    bit [15:0]       lit;
    bit [MW-1:0]     mp;
    bit [LW-1:0]     lp;
    bit [pk::PW-1:0] pp;
    bit [UW-1:0]     up;
    bit [CW-1:0]     cp;
  endclass
  int w_lit, w_mp, w_lp, w_pp, w_up, w_cp;
  initial begin
    c1 o = new();
    w_lit = $bits(o.lit);
    w_mp  = $bits(o.mp);
    w_lp  = $bits(o.lp);
    w_pp  = $bits(o.pp);
    w_up  = $bits(o.up);
    w_cp  = $bits(o.cp);
  end
endmodule
"#;

/// Two specializations of one parameterized class coexist, each sizing its own
/// properties. A width of 1 would also TRUNCATE the payload, so the stored
/// values are checked alongside `$bits` — that is the part a `$bits`-only
/// assertion would miss.
const PER_SPEC: &str = r#"
module tb;
  class box #(parameter int W = 8);
    bit [W-1:0]   data;
    bit [2*W-1:0] wide;
    function new(bit [W-1:0] d = '0);
      data = d;
    endfunction
    function bit [W-1:0] get();
      return data;
    endfunction
    function int inner_bits();
      return $bits(data);
    endfunction
  endclass
  int b16_bits, b8_bits, bd_bits, b16_inner, b16_wide_bits;
  int b16_data, b8_data, bd_data, b16_get, b16_wide;
  initial begin
    box #(16) b16;
    box #(8)  b8;
    box      bd;
    b16 = new(16'hABCD);
    b8  = new(8'h5A);
    bd  = new(8'hFF);
    b16.wide = 32'hDEADBEEF;

    b16_bits      = $bits(b16.data);
    b8_bits       = $bits(b8.data);
    bd_bits       = $bits(bd.data);
    b16_inner     = b16.inner_bits();
    b16_wide_bits = $bits(b16.wide);

    b16_data = b16.data;
    b8_data  = b8.data;
    bd_data  = bd.data;
    b16_get  = b16.get();
    b16_wide = b16.wide;
  end
endmodule
"#;

/// A range that mixes a class parameter with an enclosing-scope parameter.
/// The instance carries only the CLASS parameters, so the per-instance
/// re-resolve cannot evaluate `MW` and must fall back to the width elaboration
/// computed (which had both in scope) rather than silently dropping the
/// dimension and yielding 1.
const MIXED_SCOPE: &str = r#"
module tb;
  parameter int MW = 4;
  class mix #(parameter int W = 8);
    bit [W+MW-1:0] both;
    bit [W-1:0]    only_class;
  endclass
  int w_both, w_class, v_both;
  initial begin
    mix #(8) m;
    m = new();
    m.both = 12'hABC;
    w_both  = $bits(m.both);
    w_class = $bits(m.only_class);
    v_both  = m.both;
  end
endmodule
"#;

/// A parameterized class used as a base: the derived class's own properties
/// size from its own parameter, and the inherited property keeps the width the
/// base was elaborated with.
const DERIVED: &str = r#"
module tb;
  class base #(parameter int BW = 4);
    bit [BW-1:0] bdata;
  endclass
  class derived #(parameter int DW = 12) extends base;
    bit [DW-1:0] ddata;
  endclass
  int d_own, d_inherited;
  initial begin
    derived #(24) d;
    d = new();
    d.ddata = 24'hFFFFFF;
    d_own = $bits(d.ddata);
    d_inherited = $bits(d.bdata);
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

#[test]
fn class_property_range_resolves_every_parameter_scope() {
    let sim = simulate(SCOPES, 100).expect("simulate failed");
    assert_eq!(u(&sim, "w_lit"), 16, "literal range (never broken)");
    assert_eq!(
        u(&sim, "w_mp"),
        12,
        "module parameter in a class property range"
    );
    assert_eq!(
        u(&sim, "w_lp"),
        10,
        "module localparam in a class property range"
    );
    assert_eq!(
        u(&sim, "w_pp"),
        20,
        "package parameter in a class property range"
    );
    assert_eq!(
        u(&sim, "w_up"),
        24,
        "$unit parameter in a class property range"
    );
    assert_eq!(
        u(&sim, "w_cp"),
        6,
        "class-body localparam in a class property range"
    );
}

#[test]
fn each_specialization_sizes_its_own_properties() {
    let sim = simulate(PER_SPEC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "b16_bits"), 16, "box#(16) sizes data to its own W");
    assert_eq!(u(&sim, "b8_bits"), 8, "box#(8) sizes data to its own W");
    assert_eq!(
        u(&sim, "bd_bits"),
        8,
        "unspecialized box uses the W default"
    );
    assert_eq!(
        u(&sim, "b16_inner"),
        16,
        "$bits inside a method sees the same width"
    );
    assert_eq!(
        u(&sim, "b16_wide_bits"),
        32,
        "a range expression over W (2*W-1:0)"
    );
}

/// A too-narrow property does not merely mis-report `$bits` — it truncates on
/// store. These are the assertions that fail loudest if the width regresses.
#[test]
fn specialization_width_is_not_truncated_on_store() {
    let sim = simulate(PER_SPEC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "b16_data"),
        0xABCD,
        "constructor argument stored full width"
    );
    assert_eq!(u(&sim, "b8_data"), 0x5A, "box#(8) keeps its 8-bit payload");
    assert_eq!(
        u(&sim, "bd_data"),
        0xFF,
        "default specialization keeps 8 bits"
    );
    assert_eq!(u(&sim, "b16_get"), 0xABCD, "method return is not narrowed");
    assert_eq!(
        u(&sim, "b16_wide"),
        0xDEAD_BEEF,
        "2*W-1:0 property holds 32 bits"
    );
}

/// Guards the fallback in `respec_packed_width`: a partially-resolvable range
/// must keep the elaborated width, not collapse to 1.
#[test]
fn range_mixing_class_and_scope_parameters_keeps_elaborated_width() {
    let sim = simulate(MIXED_SCOPE, 100).expect("simulate failed");
    assert_eq!(u(&sim, "w_both"), 12, "W+MW-1:0 with W=8, MW=4");
    assert_eq!(
        u(&sim, "w_class"),
        8,
        "the class-only range still re-resolves"
    );
    assert_eq!(
        u(&sim, "v_both"),
        0xABC,
        "a 12-bit payload is not truncated"
    );
}

#[test]
fn derived_class_sizes_own_and_inherited_properties() {
    let sim = simulate(DERIVED, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "d_own"),
        24,
        "derived#(24) sizes its own property from DW"
    );
    assert_eq!(
        u(&sim, "d_inherited"),
        4,
        "inherited property keeps the base's width"
    );
}
