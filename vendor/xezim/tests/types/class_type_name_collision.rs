//! §8.3, §6.18 — a typedef declared inside a class is scoped to that class
//! (issue #307, reported by Ganesh-TNL).
//!
//! Two classes each declaring a typedef `r` with different packed-struct
//! layouts shared one entry keyed by the bare name, so the later class's
//! layout replaced the earlier one's. A `rand` member of the first class's
//! `r` was then read through the wrong struct: its fields read x, every
//! trial failed `c_sel`, and `randomize()` exhausted its budget and
//! returned 0. This is the class-scoped residual of #300.
//!
//! Expected values come from the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

fn t_lines(src: &str) -> Vec<String> {
    lines(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// The issue's reproducer: `leaf_cfg::r` is randomized while a later,
/// never-instantiated `other_cfg` declares its own `r`.
#[test]
fn issue_307_same_typedef_name_in_two_classes() {
    let out = lines(include_str!("issue_307_class_typedef_collision.sv"));
    assert!(
        out.iter().any(|m| m.starts_with("MWE-BUDGET: PASS")),
        "{out:?}"
    );
    assert!(
        !out.iter().any(|m| m.contains("budget exhausted")),
        "{out:?}"
    );
}

/// `$bits(leaf_cfg::r)` against `$bits(other_cfg::r)`, inside and outside the classes.
#[test]
fn typedefs_of_two_classes_by_scope() {
    let src = r#"
class leaf_cfg;
  typedef struct packed { bit [1:0] intlv; bit [3:0] num_hosts; bit [3:0] first_host; } r;
  rand r regs_r;
  function int w(); return $bits(r); endfunction
  function int wm(); return $bits(regs_r); endfunction
endclass
class other_cfg;
  typedef struct packed { bit [7:0] other_field; bit [15:0] another; } r;
  r m;
  function int w(); return $bits(r); endfunction
  function int wm(); return $bits(m); endfunction
endclass
module top;
  initial begin
    leaf_cfg a = new; other_cfg b = new;
    a.regs_r = '1; a.regs_r.num_hosts = 4'h5;
    b.m = '0; b.m.another = 16'hBEEF;
    $display("T|bits %0d %0d %0d %0d", $bits(leaf_cfg::r), $bits(other_cfg::r), a.w(), b.w());
    $display("T|mem %0d %0d %h %h %h", a.wm(), b.wm(), a.regs_r, a.regs_r.num_hosts, b.m.another);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|bits 10 24 10 24", "T|mem 10 24 35f 5 beef"]
    );
}

/// A typedef of a nested class, of its outer class and of another class.
#[test]
fn nested_class_typedef() {
    let src = r#"
class outer;
  class inner;
    typedef struct packed { bit [2:0] a; bit [4:0] b; } r;
    r v;
    function int w(); return $bits(r); endfunction
  endclass
  typedef logic [11:0] r;
  r o;
endclass
class other;
  typedef struct packed { bit [15:0] x; } r;
  r m;
endclass
module top;
  initial begin
    outer::inner i = new; outer t = new; other o = new;
    i.v = '0; i.v.b = 5'h1F;
    $display("T|nest %0d %0d %0d %0d %h", i.w(), $bits(i.v), $bits(t.o), $bits(o.m), i.v);
    $display("T|q %0d %0d %0d", $bits(outer::inner::r), $bits(outer::r), $bits(other::r));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|nest 8 8 12 16 1f", "T|q 8 12 16"]);
}

/// A typedef sized by a class parameter (§8.25), next to another class's `r`.
#[test]
fn parameterized_class_typedef() {
    let src = r#"
class P #(int W = 4);
  typedef logic [W-1:0] r;
  r v;
  function int w(); return $bits(r); endfunction
endclass
class Q;
  typedef struct packed { bit [9:0] f; } r;
  r v;
  function int w(); return $bits(r); endfunction
endclass
module top;
  initial begin
    P #(6) p6 = new; P p4 = new; Q q = new;
    $display("T|param %0d %0d %0d %0d %0d %0d", p6.w(), $bits(p6.v), p4.w(), $bits(p4.v), q.w(), $bits(q.v));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|param 6 6 4 4 10 10"]);
}

/// A base and a derived class each declaring `r`; a derived class that inherits the base's.
#[test]
fn base_and_derived_typedefs() {
    let src = r#"
class base;
  typedef struct packed { bit [3:0] lo; } r;
  r bv;
  function int bw(); return $bits(r); endfunction
endclass
class derived extends base;
  typedef struct packed { bit [7:0] hi; bit [7:0] lo; } r;
  r dv;
  function int dw(); return $bits(r); endfunction
endclass
class derived2 extends base;
  r iv;
  function int iw(); return $bits(r); endfunction
endclass
module top;
  initial begin
    derived d = new; derived2 e = new;
    d.dv = '0; d.dv.hi = 8'hA5; d.bv = '1;
    $display("T|inh %0d %0d %0d %0d %h %h", d.bw(), d.dw(), $bits(d.bv), $bits(d.dv), d.dv, d.bv);
    $display("T|inh2 %0d %0d %0d", e.iw(), $bits(e.iv), $bits(derived::r));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|inh 4 16 4 16 a500 f", "T|inh2 4 4 16"]);
}

/// Same-named enum typedefs (and aliases of them) in two classes.
#[test]
fn class_enum_typedefs() {
    let src = r#"
class A;
  typedef enum logic [1:0] {RED=1, GRN=2} col;
  col c = GRN;
  typedef col alias_t;
  alias_t ac = RED;
  function string nm(); return c.name(); endfunction
endclass
class B;
  typedef enum logic [3:0] {GRN=7, RED=9, BLU=3} col;
  col c = RED;
  typedef col alias_t;
  alias_t ac = BLU;
  function string nm(); return c.name(); endfunction
endclass
module top;
  initial begin
    A a = new; B b = new;
    $display("T|en %0d %0d %s %s %0d %0d", a.c, b.c, a.nm(), b.nm(), a.ac, b.ac);
    $display("T|eb %0d %0d %0d %0d", $bits(A::col), $bits(B::col), $bits(a.ac), $bits(b.ac));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|en 2 9 GRN RED 1 3", "T|eb 2 4 2 4"]);
}

/// Two classes each declaring a nested class `N`.
#[test]
fn nested_classes_of_one_name() {
    let src = r#"
class A;
  class N; int v = 1; function int id(); return 10; endfunction endclass
  N n;
  function new(); n = new; endfunction
endclass
class B;
  class N; int v = 2; logic [7:0] w = 8'h33; function int id(); return 20; endfunction endclass
  N n;
  function new(); n = new; endfunction
endclass
module top;
  initial begin
    A a = new; B b = new;
    A::N an = new; B::N bn = new;
    $display("T|nested %0d %0d %0d %0d", a.n.v, b.n.v, a.n.id(), b.n.id());
    $display("T|q %0d %0d %0d %0d", an.v, bn.v, an.id(), bn.id());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|nested 1 2 10 20", "T|q 1 2 10 20"]);
}

/// A class typedef beside a package and a `$unit` typedef of the same name.
#[test]
fn class_package_unit_typedefs() {
    let src = r#"
package pk;
  typedef logic [5:0] r;
  class K;
    typedef logic [13:0] r;
    r v;
    function int w(); return $bits(r); endfunction
  endclass
endpackage
typedef logic [2:0] r;
class U;
  r u;
  typedef logic [20:0] t;
  t tt;
endclass
class V;
  typedef logic [7:0] t;
  t tt;
endclass
module top;
  import pk::*;
  initial begin
    K k = new; U u = new; V v = new;
    $display("T|mix %0d %0d %0d %0d %0d %0d", k.w(), $bits(k.v), $bits(pk::r), $bits(u.u), $bits(u.tt), $bits(v.tt));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|mix 14 14 6 3 21 8"]);
}

/// Each class randomizes its own `r` layout.
#[test]
fn randomize_two_class_layouts() {
    let src = r#"
class C1;
  typedef struct packed { bit [1:0] a; bit [3:0] b; } r;
  rand r x;
  constraint c { x.b == 4'h9; x.a == 2'd1; }
endclass
class C2;
  typedef struct packed { bit [11:0] b; bit [3:0] a; } r;
  rand r x;
  constraint c { x.b == 12'hABC; x.a == 4'h7; }
endclass
module top;
  initial begin
    C1 c1 = new; C2 c2 = new; int r1, r2;
    r1 = c1.randomize(); r2 = c2.randomize();
    $display("T|rand %0d %0d %h %h %h %h", r1, r2, c1.x.b, c1.x.a, c2.x.b, c2.x.a);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|rand 1 1 9 1 abc 7"]);
}

/// Out-of-block methods, `pkg::C::r`, casts, arrays, a class-handle typedef and `Base::r` from a derived class.
#[test]
fn qualified_and_out_of_block_uses() {
    let src = r#"
package pk;
  class K;
    typedef struct packed { bit [5:0] a; bit [1:0] b; } r;
    r v;
    static r sv;
    extern function r mk(bit [5:0] x);
    extern function int w(r p);
  endclass
  function K::r K::mk(bit [5:0] x); r t; t = '0; t.a = x; return t; endfunction
  function int K::w(r p); r q[$]; q.push_back(p); return $bits(q[0]) + q.size(); endfunction
endpackage
class L;
  typedef struct packed { bit [11:0] a; } r;
  r arr[2];
  function r cast_it(int x); return r'(x); endfunction
endclass
class M;
  typedef L h_t;
  typedef bit [2:0] r;
  h_t h;
  r m;
  function new(); h = new; endfunction
endclass
class N extends L;
  L::r n;
  function int wn(); return $bits(n) + $bits(L::r); endfunction
endclass
module top;
  initial begin
    pk::K k = new; L l = new; M m = new; N n = new;
    k.v = k.mk(6'h2A); pk::K::sv = k.v;
    $display("T|pk %h %h %0d %0d %0d", k.v, pk::K::sv, k.w(k.v), $bits(pk::K::r), $bits(k.v));
    l.arr[1] = l.cast_it(32'h1234);
    $display("T|L %h %0d %0d", l.arr[1], $bits(l.arr[0]), $bits(L::r));
    $display("T|M %0d %0d %0d", $bits(m.m), $bits(M::r), $bits(m.h.arr[0]));
    $display("T|N %0d", n.wn());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|pk a8 a8 9 8 8", "T|L 234 12 12", "T|M 3 3 12", "T|N 24"]
    );
}

/// §8.25/§6.18: a typedef of a class TYPE parameter (`typedef T payload_t;`)
/// follows each specialization.
#[test]
fn typedef_of_a_type_parameter() {
    let src = r#"
class G #(type T = int);
  typedef T payload_t;
  payload_t p;
  T d;
  function int w(); return $bits(payload_t); endfunction
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new;
    $display("T|tp %0d %0d %0d %0d %0d %0d", $bits(g.p), $bits(h.p), $bits(g.d), $bits(h.d), g.w(), h.w());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|tp 8 64 8 64 8 64"]);
}

/// §8.25/§6.18: the type-parameter typedef also sizes stores, method
/// locals and a static method's return value per specialization; the
/// default specialization keeps the default type.
#[test]
fn typedef_of_a_type_parameter_stores_and_returns() {
    let src = r#"
class G #(type T = int);
  typedef T payload_t;
  payload_t p;
  function int lw(); payload_t t; t = 300; return t; endfunction
  static function payload_t mk(); return 300; endfunction
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new; G gd = new;
    g.p = 300; h.p = 300; gd.p = 300;
    $display("T|st %0d %0d %0d %0d", g.p, h.p, gd.p, $bits(gd.p));
    $display("T|lw %0d %0d", g.lw(), h.lw());
    $display("T|mk %0d %0d %0d", G#(byte)::mk(), G#(longint)::mk(), $bits(G#(byte)::mk()));
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|st 44 300 300 32", "T|lw 44 300", "T|mk 44 300 8"]
    );
}

/// §7.4/§8.25: packed and unpacked array typedefs of a type parameter, and
/// a typedef sized by a value parameter, follow each specialization.
#[test]
fn array_typedefs_of_class_parameters() {
    let src = r#"
class A #(type T = int);
  typedef T ua_t[3];
  ua_t u;
endclass
class P #(type T = logic [3:0]);
  typedef T [1:0] pk_t;
  pk_t k;
  function int wk(); return $bits(pk_t); endfunction
endclass
class V #(int W = 8);
  typedef logic [W-1:0] word_t;
  word_t x;
  function int ww(); return $bits(word_t); endfunction
endclass
module top;
  initial begin
    A #(byte) a = new; A #(longint) b = new;
    P #(logic [5:0]) pa = new; P pd = new;
    V #(12) v = new; V vd = new;
    a.u[0] = 300; b.u[0] = 300;
    $display("T|ua %0d %0d %0d %0d", $bits(a.u), $bits(b.u), a.u[0], b.u[0]);
    $display("T|pk %0d %0d %0d %0d", $bits(pa.k), $bits(pd.k), pa.wk(), pd.wk());
    v.x = '1; vd.x = '1;
    $display("T|vw %0d %0d %0d %0d %0d %0d", $bits(v.x), $bits(vd.x), v.ww(), vd.ww(), v.x, vd.x);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|ua 24 192 44 300",
            "T|pk 12 8 12 8",
            "T|vw 12 8 12 8 4095 255"
        ]
    );
}

/// §7.10.2.2: `push_back` onto a queue member stores the value converted to
/// the element type, as an indexed store does.
#[test]
fn queue_member_push_truncates_to_element_type() {
    let src = r#"
class A;
  byte q[$];
endclass
class B #(type T = int);
  typedef T q_t[$];
  q_t q;
  function void f(); q.push_back(300); endfunction
endclass
module top;
  initial begin
    A a = new; B #(byte) b = new; B #(byte) c = new;
    a.q.push_back(300); b.q.push_back(300); c.f(); c.q.push_front(302);
    $display("T|q %0d %0d %0d %0d %0d", a.q[0], b.q[0], c.q[0], c.q[1], $bits(b.q));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|q 44 44 46 44 8"]);
}

/// §8.25/§8.13: a derived class sees an inherited type-parameter typedef
/// through its `extends` arguments, a qualified `G#(byte)::payload_t`
/// names that specialization's type, and typedefs of the typedef, casts
/// to it and typedefs in a derived class follow along.
#[test]
fn inherited_qualified_and_derived_typedefs() {
    let src = r#"
class G #(type T = int);
  typedef T payload_t;
  typedef payload_t [1:0] pair_t;
  payload_t p;
  pair_t pr;
  function int cv(); return payload_t'(300); endfunction
endclass
class D extends G#(shortint);
  payload_t q;
  function int wq(); return $bits(payload_t); endfunction
endclass
class E #(type U = byte) extends G#(U);
  payload_t r;
endclass
class D2 extends G#(byte);
  typedef payload_t [2:0] tri_t;
  tri_t t3;
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new; D d = new; E #(bit [2:0]) e = new; D2 d2 = new;
    G#(byte)::payload_t x; G#(shortint)::payload_t y;
    d.q = 100000; d.p = 100000;
    $display("T|der %0d %0d %0d %0d %0d", $bits(d.q), $bits(d.p), d.wq(), d.q, d.p);
    e.r = 13; e.p = 13;
    $display("T|der2 %0d %0d %0d %0d", $bits(e.r), $bits(e.p), e.r, e.p);
    x = 300; y = 100000;
    $display("T|q %0d %0d %0d %0d", $bits(x), $bits(y), x, y);
    $display("T|pr %0d %0d %0d %0d %0d", $bits(g.pr), $bits(h.pr), g.cv(), h.cv(), $bits(d2.t3));
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|der 16 16 16 -31072 -31072",
            "T|der2 3 3 5 5",
            "T|q 8 16 44 -31072",
            "T|pr 16 128 44 300 24"
        ]
    );
}

/// §8.25: `typedef G#(T) self_t;` names the enclosing specialization; an
/// object a method constructs through it is of that specialization.
#[test]
fn self_typedef_built_in_a_method() {
    let src = r#"
class G #(type T = int);
  typedef T payload_t;
  typedef G#(T) self_t;
  payload_t p;
  self_t nxt;
  function void mk(); nxt = new; endfunction
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new;
    g.mk(); h.mk();
    $display("T|self %0d %0d", $bits(g.nxt.p), $bits(h.nxt.p));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|self 8 64"]);
}

/// §8.25: `obj.m = new` for a member declared `G#(T)` builds the
/// specialization `T` is bound to on `obj`.
#[test]
fn self_typedef_built_from_outside() {
    let src = r#"
class G #(type T = int);
  typedef G#(T) self_t;
  T p;
  self_t nxt;
endclass
module top;
  initial begin
    G #(byte) g = new;
    g.nxt = new;
    $display("T|self %0d", $bits(g.nxt.p));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|self 8"]);
}

/// §7.2/§8.25: a packed struct typedef with a member of a type parameter
/// is sized by each specialization.
#[test]
fn struct_typedef_with_a_type_parameter_member() {
    let src = r#"
class G #(type T = int);
  typedef struct packed { T a; bit b; } s_t;
  s_t s;
  function int ws(); return $bits(s_t); endfunction
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new;
    $display("T|st %0d %0d %0d %0d", $bits(g.s), $bits(h.s), g.ws(), h.ws());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|st 9 65 9 65"]);
}

/// §8.22/§8.25: a class nested in a parameterized class sees the outer
/// specialization's type-parameter typedef.
#[test]
fn nested_class_uses_outer_type_parameter_typedef() {
    let src = r#"
class G #(type T = int);
  typedef T payload_t;
  class N;
    payload_t x;
    function int wn(); return $bits(payload_t); endfunction
  endclass
  N n;
  function new(); n = new; endfunction
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new;
    g.n.x = 300; h.n.x = 300;
    $display("T|nest %0d %0d %0d %0d %0d %0d", $bits(g.n.x), $bits(h.n.x), g.n.wn(), h.n.wn(), g.n.x, h.n.x);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|nest 8 64 8 64 44 300"]);
}

/// §18.4/§8.25: a `rand` member of a type-parameter typedef draws within
/// the bound type, constrained or not.
#[test]
fn randomize_a_type_parameter_typedef_member() {
    let src = r#"
class R #(type T = int);
  typedef T pl_t;
  rand pl_t r;
  rand pl_t c;
  constraint k { c > 5; c < 10; }
endclass
module top;
  initial begin
    R #(bit [3:0]) a = new; R #(bit [40:0]) b = new;
    int unsigned mx = 0; int ok = 1;
    longint unsigned bmx = 0;
    repeat (40) begin
      if (!a.randomize()) ok = 0;
      if (a.r > mx) mx = a.r;
      if (!(a.c > 5 && a.c < 10)) ok = 0;
      if (!b.randomize()) ok = 0;
      if (b.r > bmx) bmx = b.r;
    end
    $display("T|rnd %0d %0d %0d %0d %0d", ok, mx < 16, mx > 7, bmx >= 64'd4294967296, $bits(a.r));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|rnd 1 1 1 1 4"]);
}

/// §8.25: `obj.m = new` builds the specialization `obj` binds for a member
/// declared through a typedef of the class (`self_t`), through `G#(T)`
/// directly, or through a value parameter, at any depth of the handle chain.
#[test]
fn members_of_a_specialization_built_from_outside() {
    let src = r#"
class G #(type T = int);
  typedef G#(T) self_t;
  T p;
  self_t nxt;
  G#(T) dn;
  function void mk(); nxt = new; dn = new; endfunction
endclass
class H #(int W = 4);
  typedef H#(W) self_t;
  bit [W-1:0] p;
  self_t nxt;
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new; G #(shortint) k = new; H #(12) hh = new;
    g.nxt = new; g.dn = new; h.nxt = new();
    g.nxt.nxt = new;
    k.mk(); k.nxt.nxt = new;
    hh.nxt = new;
    $display("T|self %0d %0d %0d %0d", $bits(g.nxt.p), $bits(g.dn.p), $bits(h.nxt.p), $bits(g.nxt.nxt.p));
    $display("T|self2 %0d %0d %0d %0d", $bits(k.nxt.p), $bits(k.dn.p), $bits(k.nxt.nxt.p), $bits(hh.nxt.p));
    g.nxt.p = 300; h.nxt.p = 300; hh.nxt.p = '1;
    $display("T|val %0d %0d %0d", g.nxt.p, h.nxt.p, hh.nxt.p);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|self 8 8 64 8",
            "T|self2 16 16 16 12",
            "T|val 44 300 4095"
        ]
    );
}

/// §7.2/§8.25: packed structs and unions with type-parameter members —
/// nested, in a fixed array, as a method local, and the default
/// specialization — follow each specialization, and their fields store at
/// the bound width.
#[test]
fn packed_aggregates_of_a_type_parameter() {
    let src = r#"
class G #(type T = int);
  typedef struct packed { T a; bit b; } s_t;
  typedef struct packed { s_t in; T c; } s2_t;
  typedef union packed { T a; T b; } u_t;
  s_t s;
  s2_t s2;
  u_t u;
  s_t sa[2];
  function int ws(); return $bits(s_t); endfunction
  function int wl(); s_t l; l = '1; return $bits(l); endfunction
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new; G gd = new;
    $display("T|st %0d %0d %0d %0d %0d", $bits(g.s), $bits(h.s), g.ws(), h.ws(), $bits(gd.s));
    $display("T|st2 %0d %0d %0d %0d %0d", $bits(g.s2), $bits(h.s2), $bits(g.u), $bits(h.u), g.wl());
    g.s.a = 300; g.s.b = 1; h.s.a = 300;
    $display("T|v %0d %0d %0d %h %0d", g.s.a, g.s.b, h.s.a, g.s, $bits(g.sa));
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|st 9 65 9 65 33",
            "T|st2 17 129 8 64 9",
            "T|v 44 1 300 059 18"
        ]
    );
}

/// §8.23/§8.25: a nested class sees the enclosing specialization's type
/// parameter directly, and a value parameter sizes its members too.
#[test]
fn nested_class_uses_outer_parameters() {
    let src = r#"
class G #(type T = int);
  typedef T payload_t;
  class N;
    payload_t x;
    T y;
    function int wn(); return $bits(payload_t); endfunction
  endclass
  N n;
  function new(); n = new; endfunction
endclass
class V #(int W = 4);
  class M;
    bit [W-1:0] z;
  endclass
  M m;
  function new(); m = new; endfunction
endclass
module top;
  initial begin
    G #(byte) g = new; G #(longint) h = new; V #(12) v = new; V vd = new;
    g.n.x = 300; h.n.x = 300; g.n.y = 300;
    $display("T|nest %0d %0d %0d %0d %0d %0d", $bits(g.n.x), $bits(h.n.x), g.n.wn(), h.n.wn(), g.n.x, h.n.x);
    $display("T|nest2 %0d %0d %0d %0d", $bits(g.n.y), g.n.y, $bits(v.m.z), $bits(vd.m.z));
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|nest 8 64 8 64 44 300", "T|nest2 8 44 12 4"]
    );
}
