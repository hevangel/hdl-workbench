//! §7.2 / §18.4 — UNPACKED-struct CLASS PROPERTIES, and `$bits` of the same
//! types. Reference-validated.
//!
//! Member resolution for a struct property went one level below the property
//! and no further, keying `<prop>.<member>` in the instance's cell map. So
//! `o.n.inner.a` matched no cell and fell through to a module-scope name — and
//! since that name is per-DESIGN rather than per-object, two instances shared
//! it: writing `b.n.inner.a` changed what `a.n.inner.a` read. That is the
//! dangerous shape — a silent cross-object leak with nothing wrong at the
//! write site. An indexed leaf (`o.s.arr[1]`, `o.arr[0].a`) was lost outright,
//! and a whole struct property read as a value (`return s;`, `v = o.s;`)
//! returned the property's unused scalar cell instead of its members.
//!
//! Separately, `$bits` of a struct ignored a member's unpacked dimensions, so
//! `struct { logic [7:0] tag; logic [7:0] arr [3]; }` reported 16 instead of
//! 32. That width also sized the variable, which is how a returned struct got
//! truncated and lost the members laid out above the cut.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Two objects must not share a nested property leaf.
#[test]
fn nested_struct_properties_are_per_instance() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; } s_t;
typedef struct { s_t inner; logic [7:0] z; }     n_t;
class C;
  n_t n;
endclass
module tb;
  C c1, c2;
  int v1, v2;
  initial begin
    c1 = new(); c2 = new();
    c1.n.inner.a = 8'h21;
    c2.n.inner.a = 8'h71;
    v1 = c1.n.inner.a;
    v2 = c2.n.inner.a;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "v1"), 0x21, "each object keeps its own nested leaf");
    assert_eq!(u(&sim, "v2"), 0x71);
}

/// Every depth and shape, written and read both inside and outside the class.
#[test]
fn struct_property_leaves_at_every_depth() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; }         s_t;
typedef struct { s_t inner; logic [7:0] z; }             n_t;
typedef struct { logic [7:0] tag; logic [7:0] arr [3]; } am_t;
class C;
  n_t  n;
  am_t am;
  s_t  arr [2];
  int  seen_n, seen_this, seen_am, seen_arr;
  function void fill();
    n.inner.a = 8'h21;
    am.arr[1] = 8'h32;
    arr[0].a  = 8'h41;
  endfunction
  function void observe();
    seen_n    = n.inner.a;
    seen_this = this.n.inner.a;
    seen_am   = am.arr[1];
    seen_arr  = arr[0].a;
  endfunction
endclass
module tb;
  C ci, co;
  int i_n, i_am, i_arr, o_n, o_am, o_arr;
  int ci_n, ci_this, ci_am, ci_arr, co_n2, co_am2, co_arr2;
  initial begin
    ci = new(); ci.fill(); ci.observe();
    co = new();
    co.n.inner.a = 8'h71;
    co.am.arr[1] = 8'h72;
    co.arr[0].a  = 8'h73;
    co.observe();
    #1;
    i_n = ci.n.inner.a; i_am = ci.am.arr[1]; i_arr = ci.arr[0].a;
    o_n = co.n.inner.a; o_am = co.am.arr[1]; o_arr = co.arr[0].a;
    ci_n = ci.seen_n; ci_this = ci.seen_this; ci_am = ci.seen_am; ci_arr = ci.seen_arr;
    co_n2 = co.seen_n; co_am2 = co.seen_am; co_arr2 = co.seen_arr;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "i_n"), u(&sim, "i_am"), u(&sim, "i_arr")),
        (0x21, 0x32, 0x41),
        "written inside, read outside"
    );
    assert_eq!(
        (u(&sim, "ci_n"), u(&sim, "ci_this")),
        (0x21, 0x21),
        "read inside, implicit and explicit this"
    );
    assert_eq!(
        (u(&sim, "ci_am"), u(&sim, "ci_arr")),
        (0x32, 0x41),
        "indexed leaves read inside"
    );
    assert_eq!(
        (u(&sim, "o_n"), u(&sim, "o_am"), u(&sim, "o_arr")),
        (0x71, 0x72, 0x73),
        "written outside, read outside"
    );
    assert_eq!(
        (u(&sim, "co_n2"), u(&sim, "co_am2"), u(&sim, "co_arr2")),
        (0x71, 0x72, 0x73),
        "written outside, read inside"
    );
}

/// A whole struct property as a value: copied out, and returned from a method.
#[test]
fn whole_struct_property_as_a_value() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; } s_t;
class C;
  s_t s;
  function void fill(); s.a = 8'h11; s.b = 8'h12; endfunction
  function automatic s_t ret_prop();     return s;                endfunction
  function automatic s_t ret_implicit(); ret_implicit.a = 8'h21;  endfunction
  function automatic s_t ret_local();    s_t t; t.a = 8'h31; return t; endfunction
endclass
module tb;
  C c;
  s_t direct, rp, ri, rl;
  int d_a, d_b, p_a, i_a, l_a;
  initial begin
    c = new(); c.fill();
    direct = c.s;
    rp = c.ret_prop();
    ri = c.ret_implicit();
    rl = c.ret_local();
    #1;
    d_a = direct.a; d_b = direct.b; p_a = rp.a; i_a = ri.a; l_a = rl.a;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "d_a"), u(&sim, "d_b")),
        (0x11, 0x12),
        "whole property copied out"
    );
    assert_eq!(u(&sim, "p_a"), 0x11, "method returning a property");
    assert_eq!(
        u(&sim, "i_a"),
        0x21,
        "method returning via the implicit variable"
    );
    assert_eq!(u(&sim, "l_a"), 0x31, "method returning a local");
}

/// `$bits` counts a member's unpacked dimensions.
#[test]
fn bits_counts_member_dimensions() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; }          flat_t;
typedef struct { logic [7:0] tag; logic [7:0] arr [3]; }  arrm_t;
typedef struct { flat_t i; logic [7:0] z; }               nest_t;
typedef struct { flat_t i [2]; logic [7:0] z; }           nestarr_t;
module tb;
  flat_t f; arrm_t am; nest_t n; nestarr_t na; flat_t fa [4];
  int w_f, w_am, w_n, w_na, w_fa, w_elem, w_type;
  initial begin
    w_f = $bits(f); w_am = $bits(am); w_n = $bits(n); w_na = $bits(na);
    w_fa = $bits(fa); w_elem = $bits(fa[0]); w_type = $bits(arrm_t);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w_f"), 16);
    assert_eq!(u(&sim, "w_am"), 32, "an array member counts every element");
    assert_eq!(u(&sim, "w_n"), 24);
    assert_eq!(u(&sim, "w_na"), 40, "an array of nested structs, too");
    assert_eq!((u(&sim, "w_fa"), u(&sim, "w_elem")), (64, 16));
    assert_eq!(
        u(&sim, "w_type"),
        32,
        "the type itself, not just a variable"
    );
}

/// A member selected directly off a call result, for every struct shape.
#[test]
fn member_select_on_a_call_result() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; }         flat_t;
typedef struct { flat_t i; logic [7:0] z; }              nest_t;
typedef struct { logic [7:0] tag; logic [7:0] arr [3]; } arrm_t;
typedef struct packed { logic [7:0] a; logic [7:0] b; }  pk_t;
module tb;
  function automatic flat_t mk_flat(); mk_flat.a = 8'h11; endfunction
  function automatic nest_t mk_nest(); mk_nest.i.a = 8'h21; mk_nest.z = 8'h23; endfunction
  function automatic arrm_t mk_arrm(); mk_arrm.tag = 8'h31; mk_arrm.arr[1] = 8'h32; endfunction
  function automatic pk_t   mk_pk();   mk_pk.a = 8'h41; endfunction
  int f_a, n_a, n_z, a_tag, a_a1, p_a;
  initial begin
    f_a = mk_flat().a;
    n_a = mk_nest().i.a;   n_z  = mk_nest().z;
    a_tag = mk_arrm().tag; a_a1 = mk_arrm().arr[1];
    p_a = mk_pk().a;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "f_a"), 0x11, "flat");
    assert_eq!(
        (u(&sim, "n_a"), u(&sim, "n_z")),
        (0x21, 0x23),
        "nested member of a call result"
    );
    assert_eq!(
        (u(&sim, "a_tag"), u(&sim, "a_a1")),
        (0x31, 0x32),
        "indexed member of a call result"
    );
    assert_eq!(u(&sim, "p_a"), 0x41, "packed return is unaffected");
}
