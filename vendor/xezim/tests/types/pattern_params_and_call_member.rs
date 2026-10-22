//! Two independent gaps surfaced by one parameterized-interface testbench:
//!
//! 1. §10.9 / §6.20.2 — an ASSIGNMENT-PATTERN value for a multi-dim PACKED
//!    vector parameter (`parameter bit [1:0][7:0] M = '{8'h0F, 8'hF0}`)
//!    evaluated to 0. The generic const-eval has no type to take element
//!    widths from, and the only pattern packers covered typedef'd STRUCT
//!    types. Broken for the declared default AND the instance `#(...)`
//!    override, at module, package and interface scope alike; the index-keyed
//!    form (`'{1: v, 0: v}`) was additionally hijacked by the
//!    associative-array-literal branch, leaving the parameter x. Such
//!    parameters also had no per-element width registered, so `PARAM[i]`
//!    bit-selected.
//!
//! 2. §8.24 / §7.2.1 — member access chained on a CALL result (`make().tag`,
//!    `obj.read_payload().tag`, `mk().n.hi`) returned 0: `struct_field_layout`
//!    only describes UNPACKED structs and a method callee never matched the
//!    Ident-only lookups, so the field projection always fell through even
//!    though the call's whole value was correct. Same for element-indexing a
//!    call result returning a multi-dim packed vector (`mk_vec()[1]`).

use xezim::simulate;

/// Pattern parameter defaults and overrides across scopes and pattern forms.
const PATTERN_PARAMS: &str = r#"
module sub #(
  parameter bit [1:0][7:0] MASK = '{8'h0F, 8'hF0},
  parameter bit [1:0][7:0] KEYED = '{1: 8'h55, 0: 8'h66}
);
  bit [1:0][7:0] cap = MASK;
  bit [1:0][7:0] capk = KEYED;
endmodule
interface bus_if #(parameter bit [1:0][7:0] MASK = '{8'h0F, 8'hF0});
  bit [1:0][7:0] captured = MASK;
endinterface
module tb;
  parameter bit [1:0][7:0]      TOPMASK = '{8'hCC, 8'hDD};
  parameter bit [3:0][7:0]      DFLT    = '{default: 8'h9A};
  parameter bit [1:0][1:0][3:0] NEST    = '{'{4'hA, 4'hB}, '{4'hC, 4'hD}};
  localparam bit [1:0][7:0]     LP      = '{8'h33, 8'h44};
  sub #(.MASK('{8'hAA, 8'hBB})) s_ovr ();
  sub                           s_def ();
  bus_if #(.MASK('{8'h11, 8'h22})) i_ovr ();
  int top_whole, top_e1, dflt_v, nest_v, lp_v;
  int ovr_cap, def_cap, def_keyed, if_cap, if_e1, ovr_param_e1;
  initial begin
    #1;
    top_whole = TOPMASK;
    top_e1    = TOPMASK[1];
    dflt_v    = DFLT;
    nest_v    = NEST;
    lp_v      = LP;
    ovr_cap   = s_ovr.cap;
    def_cap   = s_def.cap;
    def_keyed = s_def.capk;
    if_cap    = i_ovr.captured;
    if_e1     = i_ovr.captured[1];
    ovr_param_e1 = s_ovr.MASK[1];
  end
endmodule
"#;

/// Member access and element indexing chained on call results, for free
/// functions, class methods, interface-class (virtual) dispatch, a
/// type-parameter return type, and nested struct members.
const CALL_MEMBER: &str = r#"
package pk;
  typedef struct packed { logic [7:0] tag; logic [31:0] val; } msg_t;
  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } nib_t;
  typedef struct packed { pk::nib_t n; logic [7:0] rest; } outer_t;
endpackage
interface class ichan #(type T = int);
  pure virtual function T read_payload();
endclass
class drv #(type T = int) implements ichan #(T);
  local T storage;
  virtual function void wr(T d); storage = d; endfunction
  virtual function T read_payload(); return storage; endfunction
endclass
module tb;
  function pk::outer_t mk_outer();
    mk_outer = '{n: '{hi: 4'hA, lo: 4'h5}, rest: 8'h7F};
  endfunction
  function bit [1:0][7:0] mk_vec();
    mk_vec = 16'hCCDD;
  endfunction
  int free_tag, meth_tag, ifc_tag, nested_hi, nested_rest, vec_e1, vec_e0;
  initial begin
    drv #(pk::msg_t)   d;
    ichan #(pk::msg_t) c;
    d = new();
    d.wr('{tag: 8'hE0, val: 32'h12345678});
    c = d;
    free_tag    = mk_outer().rest;
    meth_tag    = d.read_payload().tag;
    ifc_tag     = c.read_payload().tag;
    nested_hi   = mk_outer().n.hi;
    nested_rest = mk_outer().rest;
    vec_e1      = mk_vec()[1];
    vec_e0      = mk_vec()[0];
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
fn pattern_parameter_defaults_and_overrides_pack() {
    let sim = simulate(PATTERN_PARAMS, 100).expect("simulate failed");
    assert_eq!(u(&sim, "top_whole"), 0xCCDD, "top-scope pattern default");
    assert_eq!(
        u(&sim, "top_e1"),
        0xCC,
        "element select on a pattern parameter"
    );
    assert_eq!(
        u(&sim, "dflt_v"),
        0x9A9A_9A9A,
        "a default-keyed pattern replicates"
    );
    assert_eq!(u(&sim, "nest_v"), 0xABCD, "nested 3-D pattern");
    assert_eq!(u(&sim, "lp_v"), 0x3344, "localparam pattern");
    assert_eq!(
        u(&sim, "ovr_cap"),
        0xAABB,
        "an instance pattern override packs"
    );
    assert_eq!(
        u(&sim, "def_cap"),
        0x0FF0,
        "an instantiated pattern default packs"
    );
    assert_eq!(
        u(&sim, "def_keyed"),
        0x5566,
        "index-keyed pattern is NOT an assoc literal"
    );
    assert_eq!(
        u(&sim, "if_cap"),
        0x1122,
        "interface parameter override packs"
    );
    assert_eq!(u(&sim, "if_e1"), 0x11, "interface variable element select");
    assert_eq!(
        u(&sim, "ovr_param_e1"),
        0xAA,
        "hierarchical parameter element select"
    );
}

#[test]
fn member_access_on_call_results_projects_fields() {
    let sim = simulate(CALL_MEMBER, 100).expect("simulate failed");
    assert_eq!(u(&sim, "free_tag"), 0x7F, "free function result field");
    assert_eq!(
        u(&sim, "meth_tag"),
        0xE0,
        "class method result field (type-param return)"
    );
    assert_eq!(
        u(&sim, "ifc_tag"),
        0xE0,
        "interface-class virtual dispatch result field"
    );
    assert_eq!(
        u(&sim, "nested_hi"),
        0xA,
        "nested struct member on a call result"
    );
    assert_eq!(
        u(&sim, "nested_rest"),
        0x7F,
        "sibling member after nested access"
    );
    assert_eq!(u(&sim, "vec_e1"), 0xCC, "element select on a call result");
    assert_eq!(u(&sim, "vec_e0"), 0xDD, "element select on a call result");
}
