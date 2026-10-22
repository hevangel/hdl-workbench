//! Two gaps found auditing the July-2026 parameter fixes:
//!
//! * §10.9.1 — inside an assignment pattern, `N{expr}` replicates pattern
//!   ELEMENTS, not bits. The packer treated `'{4{8'h7E}}` as a single ordered
//!   element, left the other three slots unfilled, and bailed the whole
//!   pattern to 0.
//!
//! * §8.13 / §8.25 — `class D extends B #(8);` supplies B's value parameters,
//!   but ancestor parameters were always bound to their DECLARED DEFAULTS. An
//!   inherited property sized by a base parameter reported the default width
//!   (`extends base #(8)` → 4 bits), and once method returns were clamped per
//!   instance the same gap actively TRUNCATED the return value.

use xezim::simulate;

const REPLICATION: &str = r#"
module sub #(parameter bit [3:0][7:0] R = '{4{8'h11}});
  bit [3:0][7:0] cap = R;
endmodule
module tb;
  parameter bit [3:0][7:0]      TOP_R  = '{4{8'h7E}};
  parameter bit [1:0][7:0]      PART_R = '{2{8'hC3}};
  parameter bit [1:0][1:0][3:0] NEST_R = '{2{'{4'hA, 4'hB}}};
  sub #(.R('{4{8'h5A}})) s_ovr ();
  sub                    s_def ();
  int top_v, part_v, nest_v, ovr_v, def_v, elem_v;
  initial begin
    #1;
    top_v  = TOP_R;
    part_v = PART_R;
    nest_v = NEST_R;
    ovr_v  = s_ovr.cap;
    def_v  = s_def.cap;
    elem_v = TOP_R[2];
  end
endmodule
"#;

const EXTENDS_ARGS: &str = r#"
module tb;
  class base #(parameter int BW = 4);
    bit [BW-1:0] bdata;
    struct packed { bit [BW-1:0] f; bit [3:0] g; } bs;
    function bit [BW-1:0] bmk(); return 8'hFF; endfunction
  endclass
  class derived  extends base;        // base keeps its default
    bit [7:0] ddata;
  endclass
  class derived8 extends base #(8);   // base specialized through extends
    bit [7:0] ddata;
  endclass
  class fwd #(parameter int N = 12) extends base #(N);  // forwarded param
    bit [7:0] fdata;
  endclass
  int d_bits, d_mk, d8_bits, d8_mk, d8_store, f_bits, f_mk;
  initial begin
    derived  d  = new();
    derived8 d8 = new();
    fwd #(6) f  = new();
    d.bdata  = 8'hFF;
    d8.bdata = 8'hFF;
    f.bdata  = 8'hFF;
    d_bits   = $bits(d.bdata);
    d_mk     = d.bmk();
    d8_bits  = $bits(d8.bdata);
    d8_mk    = d8.bmk();
    d8_store = d8.bdata;
    f_bits   = $bits(f.bdata);
    f_mk     = f.bmk();
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
fn replicated_assignment_patterns_expand_to_elements() {
    let sim = simulate(REPLICATION, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "top_v"),
        0x7E7E_7E7E,
        "a 4x replication fills four elements"
    );
    assert_eq!(u(&sim, "part_v"), 0xC3C3, "two-element replication");
    assert_eq!(u(&sim, "nest_v"), 0xABAB, "replication of a nested pattern");
    assert_eq!(
        u(&sim, "ovr_v"),
        0x5A5A_5A5A,
        "replicated instance override"
    );
    assert_eq!(
        u(&sim, "def_v"),
        0x1111_1111,
        "replicated instantiated default"
    );
    assert_eq!(
        u(&sim, "elem_v"),
        0x7E,
        "element select into a replicated pattern"
    );
}

#[test]
fn extends_clause_binds_ancestor_value_parameters() {
    let sim = simulate(EXTENDS_ARGS, 100).expect("simulate failed");
    assert_eq!(u(&sim, "d_bits"), 4, "plain extends keeps the base default");
    assert_eq!(
        u(&sim, "d_mk"),
        0xF,
        "return clamped to the base default width"
    );
    assert_eq!(
        u(&sim, "d8_bits"),
        8,
        "extends base #(8) sizes the inherited property"
    );
    assert_eq!(u(&sim, "d8_mk"), 0xFF, "and the inherited method's return");
    assert_eq!(
        u(&sim, "d8_store"),
        0xFF,
        "a store is not truncated to the default"
    );
    assert_eq!(
        u(&sim, "f_bits"),
        6,
        "extends base #(N) forwards the derived parameter"
    );
    assert_eq!(
        u(&sim, "f_mk"),
        0x3F,
        "forwarded parameter clamps the return to 6 bits"
    );
}
