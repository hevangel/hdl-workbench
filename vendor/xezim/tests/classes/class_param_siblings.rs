//! Sibling gaps of the July-2026 class/parameter fixes, found by systematic
//! probing of each fixed bug's adjacent constructs:
//!
//! * §6.20.2 — a class property with a PARAMETER-sized unpacked dimension
//!   (`bit [7:0] mem [N];`) was queue-backed: indexing worked, but `size()`
//!   reported 1 and `foreach` iterated ZERO times. `mem[N]` parses as an
//!   associative dimension keyed by "type N", and class elaboration never
//!   rewrote it back nor had a parameter scope to size it. Sized now from the
//!   enclosing scope's parameters + class-body localparams. An OVERRIDABLE
//!   header value parameter is deliberately excluded (one `ElaboratedClass`
//!   records one shape) — such properties stay queue-backed.
//! * §6.20 — a STRUCT-pattern parameter value (`#(.CFG('{tag: ..}))`) was 0 at
//!   instance sites (override and instantiated default); only top-level
//!   module scope packed it.
//! * §8.25 — a class-parameter-sized RETURN type (`function bit [W-1:0]`)
//!   used whatever width the body produced; `box#(4)` returned 16 bits.
//! * §8.24 — a NESTED call chain (`o.get().rd().tag`) returned 0: the method
//!   receiver was itself a call, which the return-type resolver refused.
//!   Resolved statically from the receiver's declared return class now.
//! * §6.19 — class-body typedef ENUM members never resolved as constants
//!   (initializers and method bodies read x) for classes declared in a MODULE
//!   body: `register_class_enum_members` ran at only 3 of the ~10 class
//!   elaboration sites.
//! * §7.2 — an ANONYMOUS inline struct property (`struct packed {..} s;`) had
//!   no typedef name to look up, so `s.f` read x; the declared type retained
//!   in `property_types` now supplies the layout, with member widths resolved
//!   against the instance's class parameters.

use xezim::simulate;

const UNPACKED_DIMS: &str = r#"
module tb;
  parameter int N = 4;
  class holder;
    localparam int LP = 3;
    bit [7:0] mem [N];
    int       lparr [LP];
    bit [7:0] fixed [4];
  endclass
  int count, sum, msize, lsize, fsize, idx_ok;
  initial begin
    holder h = new();
    foreach (h.mem[i]) begin count++; h.mem[i] = 8'(i + 1); end
    foreach (h.mem[i]) sum += h.mem[i];
    h.lparr[2] = 42;
    h.fixed[3] = 8'h3C;
    msize = h.mem.size();
    lsize = h.lparr.size();
    fsize = h.fixed.size();
    idx_ok = (h.lparr[2] == 42) && (h.fixed[3] == 8'h3C);
  end
endmodule
"#;

const STRUCT_PARAM_SITES: &str = r#"
package pk;
  typedef struct packed { logic [7:0] tag; logic [31:0] val; } cfg_t;
endpackage
module sub #(parameter pk::cfg_t CFG = '{tag: 8'h11, val: 32'h22334455});
  pk::cfg_t cap = CFG;
endmodule
module tb;
  sub #(.CFG('{tag: 8'h99, val: 32'h44556677})) s_ovr ();
  sub                                            s_def ();
  longint ovr_v, def_v;
  initial begin
    #1;
    ovr_v = s_ovr.cap;
    def_v = s_def.cap;
  end
endmodule
"#;

const RETURN_WIDTH: &str = r#"
module tb;
  class box #(parameter int W = 16);
    function bit [W-1:0] mk();
      bit [15:0] raw;
      raw = 16'hBEEF;
      return raw;
    endfunction
  endclass
  int wide_v, narrow_v;
  initial begin
    box #(16) b = new();
    box #(4)  s = new();
    wide_v   = b.mk();
    narrow_v = s.mk();
  end
endmodule
"#;

const NESTED_CHAIN: &str = r#"
package pk;
  typedef struct packed { logic [7:0] tag; logic [31:0] val; } msg_t;
endpackage
module tb;
  class inner_c;
    pk::msg_t m;
    function new(); m = '{tag: 8'hE0, val: 32'h12345678}; endfunction
    function pk::msg_t rd(); return m; endfunction
  endclass
  class outer_c;
    inner_c i;
    function new(); i = new(); endfunction
    function inner_c get(); return i; endfunction
  endclass
  int chained_tag;
  initial begin
    outer_c o = new();
    chained_tag = o.get().rd().tag;
  end
endmodule
"#;

const CLASS_ENUMS: &str = r#"
module tb;
  parameter int EW = 4;
  class box;
    typedef enum bit [3:0]    { A1 = 1, B1 = 14 } lit_e;
    typedef enum bit [EW-1:0] { A2 = 1, B2 = 14 } par_e;
    typedef enum              { A3, B3, C3 }      plain_e;
    lit_e   e_lit = B1;
    par_e   e_par = B2;
    plain_e e_pln = C3;
    function void set_in_method(); e_lit = A1; endfunction
  endclass
  int v_lit, v_par, v_pln, v_meth, v_scoped, w_par;
  initial begin
    box b = new();
    v_lit = b.e_lit;
    v_par = b.e_par;
    w_par = $bits(b.e_par);
    v_pln = b.e_pln;
    b.set_in_method();
    v_meth = b.e_lit;
    v_scoped = box::B1;
  end
endmodule
"#;

const ANON_STRUCT: &str = r#"
module tb;
  class box #(parameter int W = 8);
    struct packed { bit [W-1:0] f; bit [3:0] g; } s;
  endclass
  int whole, f_v, g_v, s_bits;
  initial begin
    box b = new();
    b.s = 12'hAB5;
    whole  = b.s;
    f_v    = b.s.f;
    g_v    = b.s.g;
    s_bits = $bits(b.s);
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
fn parameter_sized_unpacked_class_arrays_are_fixed_arrays() {
    let sim = simulate(UNPACKED_DIMS, 100).expect("simulate failed");
    assert_eq!(u(&sim, "count"), 4, "foreach iterates the declared extent");
    assert_eq!(u(&sim, "sum"), 10, "elements written in foreach read back");
    assert_eq!(u(&sim, "msize"), 4, "size() from a module parameter");
    assert_eq!(u(&sim, "lsize"), 3, "size() from a class-body localparam");
    assert_eq!(u(&sim, "fsize"), 4, "literal-sized control unchanged");
    assert_eq!(u(&sim, "idx_ok"), 1, "indexed access still works");
}

#[test]
fn struct_pattern_parameters_pack_at_instance_sites() {
    let sim = simulate(STRUCT_PARAM_SITES, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "ovr_v"),
        0x99_4455_6677,
        "named struct-pattern override"
    );
    assert_eq!(
        u(&sim, "def_v"),
        0x11_2233_4455,
        "instantiated struct-pattern default"
    );
}

#[test]
fn method_return_width_follows_the_specialization() {
    let sim = simulate(RETURN_WIDTH, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "wide_v"),
        0xBEEF,
        "default-spec return keeps 16 bits"
    );
    assert_eq!(
        u(&sim, "narrow_v"),
        0xF,
        "box#(4) clamps the return to 4 bits"
    );
}

#[test]
fn nested_call_chain_projects_the_field() {
    let sim = simulate(NESTED_CHAIN, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "chained_tag"),
        0xE0,
        "o.get().rd().tag via static receiver typing"
    );
}

#[test]
fn class_body_enum_members_resolve_as_constants() {
    let sim = simulate(CLASS_ENUMS, 100).expect("simulate failed");
    assert_eq!(u(&sim, "v_lit"), 14, "literal-base enum initializer");
    assert_eq!(u(&sim, "v_par"), 14, "parameter-base enum initializer");
    assert_eq!(u(&sim, "w_par"), 4, "parameter-base enum width");
    assert_eq!(u(&sim, "v_pln"), 2, "default-base auto-increment");
    assert_eq!(u(&sim, "v_meth"), 1, "enum member assigned inside a method");
    assert_eq!(u(&sim, "v_scoped"), 14, "Class::MEMBER scoped access");
}

#[test]
fn anonymous_inline_struct_property_projects_fields() {
    let sim = simulate(ANON_STRUCT, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "whole"),
        0xAB5,
        "whole-value access (already worked)"
    );
    assert_eq!(u(&sim, "f_v"), 0xAB, "field sized by a class parameter");
    assert_eq!(u(&sim, "g_v"), 0x5, "literal-width field");
    assert_eq!(u(&sim, "s_bits"), 12, "$bits of the anonymous struct");
}
