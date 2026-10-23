//! §8.25 / §8.3: a class whose base class is a TYPE PARAMETER
//! (`class wrap_c #(type BASE = base_c) extends BASE;`), issue #210.
//!
//! The class was linked to no base at all: `extends` kept the literal
//! parameter name, which names no class, so every walk up the hierarchy
//! stopped at `wrap_c` — no base constructor, inherited properties read x
//! with their initializers skipped, inherited methods returned 0,
//! `super.who()` was empty and the upcast `$cast` failed. Each
//! specialization now links to its own bound base. Expected lines come
//! from the reference simulator.

use std::process::Command;

/// Run `src` (top `tb`) and return its `T|` lines.
fn run(tag: &str, src: &str) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("xezim_tpbase_{}_{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("t.sv");
    std::fs::write(&sv, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg("--simulate")
        .arg("-s")
        .arg("tb")
        .arg(sv.to_str().unwrap())
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.starts_with("T|"))
        .map(|l| l.trim_end().to_string())
        .collect()
}

fn check(tag: &str, src: &str, expected: &[&str]) {
    let got = run(tag, src);
    assert_eq!(
        got,
        expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "T| lines differ from the reference"
    );
}

/// The issue's reproducer: default specialization, explicit `wrap_c #(derived_c)` and a
/// two-level `wrap2_c #(B) extends wrap_c #(B)` — base constructor, inherited property and
/// initializer, inherited method, `super.who()`, `$cast` up and down, plain upcast.
#[test]
fn issue_210_type_param_base_construction_members_cast() {
    const SRC: &str = r#"
class base_c;
  int x = 5;
  function new(); $display("T| base_c::new"); endfunction
  virtual function string who(); return "base_c"; endfunction
  function int getx(); return x; endfunction
endclass
class derived_c extends base_c;
  int y = 7;
  virtual function string who(); return "derived_c"; endfunction
endclass
class wrap_c #(type BASE = base_c) extends BASE;
  function new(); super.new(); $display("T| wrap_c::new"); endfunction
  virtual function string who(); return {"wrap_c/", super.who()}; endfunction
  function int getx2(); return x * 2; endfunction
endclass
class wrap2_c #(type B = base_c) extends wrap_c #(B); endclass
module tb;
  base_c b, b2; wrap_c w, w3; wrap_c #(derived_c) wd; derived_c d; wrap2_c ww;
  int ok;
  initial begin
    w = new();
    $display("T| cast_up=%0d", $cast(b, w));
    b2 = w;
    $display("T| assign_up who=%s", b2.who());
    $display("T| cast_down=%0d", $cast(w3, b2));
    $display("T| getx=%0d getx2=%0d x=%0d who=%s", w.getx(), w.getx2(), w.x, w.who());
    w.x = 9; $display("T| x_write=%0d", w.getx());
    wd = new();
    $display("T| wd who=%s y=%0d cast_d=%0d", wd.who(), wd.y, $cast(d, wd));
    ww = new();
    $display("T| ww who=%s cast_b=%0d", ww.who(), $cast(b, ww));
    $finish;
  end
endmodule
"#;
    check(
        "t1",
        SRC,
        &[
            "T| base_c::new",
            "T| wrap_c::new",
            "T| cast_up=1",
            "T| assign_up who=wrap_c/base_c",
            "T| cast_down=1",
            "T| getx=5 getx2=10 x=5 who=wrap_c/base_c",
            "T| x_write=9",
            "T| base_c::new",
            "T| wrap_c::new",
            "T| wd who=wrap_c/derived_c y=7 cast_d=1",
            "T| base_c::new",
            "T| wrap_c::new",
            "T| ww who=wrap_c/base_c cast_b=1",
        ],
    );
}

/// Per-specialization bases: `$typename`, a typedef of the specialization, a class that
/// extends a specialization (`leaf_c extends wrap_c #(derived_c)`), a value-parameterized
/// base (`wrap_c #(pbase#(8))`), `$cast` between specializations, and per-specialization
/// statics counted through every construction path.
#[test]
fn type_param_base_specializations_typedef_leaf_statics() {
    const SRC: &str = r#"
class base_c;
  int x = 5;
  static int cnt = 0;
  string tag;
  function new(string t = "dflt"); tag = t; cnt++; $display("T| base_c::new %s", t); endfunction
  virtual function string who(); return "base_c"; endfunction
  function int getx(); return x; endfunction
  static function int get_cnt(); return cnt; endfunction
endclass
class derived_c extends base_c;
  int y = 7;
  function new(string t = "dd"); super.new(t); endfunction
  virtual function string who(); return "derived_c"; endfunction
endclass
class pbase #(int W = 4) extends base_c;
  function new(string t = "p"); super.new(t); endfunction
  virtual function string who(); return $sformatf("pbase%0d", W); endfunction
endclass
class wrap_c #(type BASE = base_c) extends BASE;
  static int wcnt = 0;
  int z = 3;
  function new(string t = "w"); super.new(t); wcnt++; $display("T| wrap_c::new"); endfunction
  virtual function string who(); return {"wrap_c/", super.who()}; endfunction
endclass
class wrap2_c #(type B = base_c) extends wrap_c #(B);
  function new(); super.new("w2"); endfunction
  virtual function string who(); return {"wrap2_c/", super.who()}; endfunction
endclass
class leaf_c extends wrap_c #(derived_c);
  function new(); super.new("leaf"); endfunction
  virtual function string who(); return {"leaf/", super.who()}; endfunction
endclass
typedef wrap_c #(derived_c) wd_t;
module tb;
  base_c b; derived_c d;
  wrap_c w; wrap_c #(derived_c) wd; wd_t wt; wrap2_c #(derived_c) w2d; leaf_c lf;
  wrap_c #(pbase#(8)) wp; pbase #(8) pb;
  wrap_c #(base_c) wb;
  initial begin
    w = new("a");
    $display("T| w tag=%s z=%0d x=%0d", w.tag, w.z, w.x);
    $display("T| tn w=%s", $typename(w));
    wd = new("b");
    $display("T| tn wd=%s", $typename(wd));
    $display("T| wd tag=%s y=%0d", wd.tag, wd.y);
    wt = new();
    $display("T| wt who=%s y=%0d", wt.who(), wt.y);
    $display("T| tn wt=%s", $typename(wt));
    w2d = new();
    $display("T| w2d who=%s y=%0d", w2d.who(), w2d.y);
    $display("T| cast_d=%0d", $cast(d, w2d));
    lf = new();
    $display("T| lf who=%s y=%0d tag=%s", lf.who(), lf.y, lf.tag);
    b = lf;
    $display("T| b who=%s", b.who());
    $display("T| cast_wd=%0d", $cast(wd, b));
    $display("T| cast_w=%0d", $cast(w, b));
    wp = new();
    $display("T| wp who=%s W=%0d", wp.who(), wp.W);
    $display("T| cast_pb=%0d", $cast(pb, wp));
    $display("T| cnt=%0d %0d wcnt=%0d wdcnt=%0d", base_c::cnt, wrap_c#(derived_c)::get_cnt(), wrap_c#()::wcnt, wrap_c#(derived_c)::wcnt);
    $display("T| wdtcnt=%0d", wd_t::wcnt);
    wb = new();
    $display("T| tn wb=%s", $typename(wb));
    $display("T| cast_wbw=%0d", $cast(w, wb));
    $finish;
  end
endmodule
"#;
    check(
        "t2",
        SRC,
        &[
            "T| base_c::new a",
            "T| wrap_c::new",
            "T| w tag=a z=3 x=5",
            "T| tn w=class wrap_c #(class base_c)",
            "T| base_c::new b",
            "T| wrap_c::new",
            "T| tn wd=class wrap_c #(class derived_c)",
            "T| wd tag=b y=7",
            "T| base_c::new w",
            "T| wrap_c::new",
            "T| wt who=wrap_c/derived_c y=7",
            "T| tn wt=class wrap_c #(class derived_c)",
            "T| base_c::new w2",
            "T| wrap_c::new",
            "T| w2d who=wrap2_c/wrap_c/derived_c y=7",
            "T| cast_d=1",
            "T| base_c::new leaf",
            "T| wrap_c::new",
            "T| lf who=leaf/wrap_c/derived_c y=7 tag=leaf",
            "T| b who=leaf/wrap_c/derived_c",
            "T| cast_wd=1",
            "T| cast_w=0",
            "T| base_c::new w",
            "T| wrap_c::new",
            "T| wp who=wrap_c/pbase8 W=8",
            "T| cast_pb=1",
            "T| cnt=6 6 wcnt=1 wdcnt=4",
            "T| wdtcnt=4",
            "T| base_c::new w",
            "T| wrap_c::new",
            "T| tn wb=class wrap_c #(class base_c)",
            "T| cast_wbw=1",
        ],
    );
}

/// `extends BASE(42)` constructor arguments, statics and static methods of the bound base
/// through an instance, `$cast` to a typedef-declared destination, and the mixin pattern
/// with a parameterized base (`mixin_c #(comp_base#(byte))`).
#[test]
fn type_param_base_ctor_args_mixin_typedef_cast() {
    const SRC: &str = r#"
class base_c;
  int x = 5;
  static int bs = 0;
  function new(int v = 1); x = v; $display("T| base_c::new %0d", v); endfunction
  virtual function string who(); return "base_c"; endfunction
  static function string sname(); return "base_s"; endfunction
endclass
class derived_c extends base_c;
  int y = 7;
  static int ds = 3;
  function new(int v = 2); super.new(v + 10); y = v; endfunction
  virtual function string who(); return "derived_c"; endfunction
  static function string dname(); return "derived_s"; endfunction
endclass
// value args to the type-param base's constructor via extends clause
class wrap_c #(type BASE = base_c) extends BASE(42);
  static int wcnt;
  function new(); wcnt++; endfunction
  virtual function string who(); return {"wrap_c/", super.who()}; endfunction
endclass
// UVM mixin: type-param base itself parameterized
class comp_base #(type T = int) ;
  T data;
  function new(); endfunction
  virtual function string who(); return $sformatf("comp_base#(%s)", $typename(T)); endfunction
endclass
class mixin_c #(type B = comp_base#(int)) extends B;
  function new(); super.new(); endfunction
  virtual function string who(); return {"mixin/", super.who()}; endfunction
endclass
typedef wrap_c #(derived_c) wd_t;
module tb;
  base_c b; derived_c d;
  wrap_c w; wd_t wt, wt2; wrap_c#(derived_c) wd;
  mixin_c m; mixin_c #(comp_base#(byte)) mb; comp_base#(byte) cb; comp_base#(int) ci;
  initial begin
    w = new();
    $display("T| w x=%0d who=%s", w.x, w.who());
    wt = new();
    $display("T| wt x=%0d y=%0d who=%s", wt.x, wt.y, wt.who());
    $display("T| tn wt=%s", $typename(wt));
    $display("T| static d: %s %s", wt.dname(), wt.sname());
    b = wt;
    $display("T| cast wt2=%0d wd=%0d", $cast(wt2, b), $cast(wd, b));
    b = w;
    $display("T| cast wt2 from w=%0d", $cast(wt2, b));
    $display("T| wcnt=%0d %0d", wrap_c#()::wcnt, wd_t::wcnt);
    m = new(); mb = new();
    $display("T| m who=%s mb who=%s", m.who(), mb.who());
    $display("T| cast cb=%0d ci=%0d", $cast(cb, mb), $cast(ci, m));
    $display("T| tn mb=%s", $typename(mb));
    mb.data = 8'hff; m.data = 32'h1ff;
    $display("T| data %0h %0h", mb.data, m.data);
    $finish;
  end
endmodule
"#;
    check(
        "t3",
        SRC,
        &[
            "T| base_c::new 42",
            "T| w x=42 who=wrap_c/base_c",
            "T| base_c::new 52",
            "T| wt x=52 y=42 who=wrap_c/derived_c",
            "T| tn wt=class wrap_c #(class derived_c)",
            "T| static d: derived_s base_s",
            "T| cast wt2=1 wd=1",
            "T| cast wt2 from w=0",
            "T| wcnt=1 1",
            "T| m who=mixin/comp_base#(int) mb who=mixin/comp_base#(byte)",
            "T| cast cb=1 ci=1",
            "T| tn mb=class mixin_c #(class comp_base #(byte))",
            "T| data ff 1ff",
        ],
    );
}

/// A static of the type-parameter class counted per specialization from `new`, a typedef,
/// a parameterized derived class and a non-parameterized derived class, read both through
/// the class scope and through instances.
#[test]
fn type_param_base_statics_per_specialization() {
    const SRC: &str = r#"
class base_c;
  static int bs;
endclass
class derived_c extends base_c;
  static int ds = 3;
endclass
class wrap_c #(type BASE = base_c) extends BASE;
  static int wcnt = 0;
  function new(); wcnt++; endfunction
endclass
class wrap2_c #(type B = base_c) extends wrap_c #(B);
  function new(); super.new(); endfunction
endclass
class leaf_c extends wrap_c #(derived_c);
  function new(); super.new(); endfunction
endclass
typedef wrap_c #(derived_c) wd_t;
module tb;
  wrap_c #(derived_c) wd; wd_t wt; wrap2_c #(derived_c) w2d; leaf_c lf; wrap_c w;
  initial begin
    wd = new(); $display("T| a %0d", wrap_c#(derived_c)::wcnt);
    wt = new(); $display("T| b %0d", wrap_c#(derived_c)::wcnt);
    w2d = new(); $display("T| c %0d", wrap_c#(derived_c)::wcnt);
    lf = new(); $display("T| d %0d %0d", wrap_c#(derived_c)::wcnt, wrap_c#()::wcnt);
    $display("T| e %0d %0d %0d %0d", wd.wcnt, wt.wcnt, w2d.wcnt, lf.wcnt);
    w = new();
    $display("T| g %0d %0d", w.wcnt, wrap_c#()::wcnt);
  end
endmodule
"#;
    check(
        "t4",
        SRC,
        &[
            "T| a 1",
            "T| b 2",
            "T| c 3",
            "T| d 4 0",
            "T| e 4 4 4 4",
            "T| g 1 1",
        ],
    );
}

/// A class extending a typedef of a specialization, a wrapper whose own default binds a
/// non-default base (`w3 #(type B = derived_c) extends wrap_c #(B)`), and class properties
/// typed by a specialization or its typedef, constructed inside a method.
#[test]
fn type_param_base_typedef_extends_defaulted_wrapper_properties() {
    const SRC: &str = r#"
class base_c;
  int x = 5;
  virtual function string who(); return "base_c"; endfunction
endclass
class derived_c extends base_c;
  int y = 7;
  virtual function string who(); return "derived_c"; endfunction
endclass
class wrap_c #(type BASE = base_c) extends BASE;
  virtual function string who(); return {"wrap_c/", super.who()}; endfunction
endclass
typedef wrap_c #(derived_c) wd_t;
class via_td extends wd_t;
  virtual function string who(); return {"via_td/", super.who()}; endfunction
endclass
class w3 #(type B = derived_c) extends wrap_c #(B);
  virtual function string who(); return {"w3/", super.who()}; endfunction
endclass
class owner;
  wrap_c #(derived_c) p;
  wd_t q;
  function void build(); p = new(); q = new(); endfunction
endclass
module tb;
  via_td v; w3 a; w3 #(base_c) ab; owner o; derived_c d; base_c b;
  initial begin
    v = new();
    $display("T| v who=%s y=%0d cast=%0d", v.who(), v.y, $cast(d, v));
    a = new(); ab = new();
    $display("T| a who=%s y=%0d", a.who(), a.y);
    $display("T| ab who=%s cast=%0d", ab.who(), $cast(d, ab));
    $display("T| tn a=%s ab=%s", $typename(a), $typename(ab));
    o = new(); o.build();
    $display("T| o.p who=%s y=%0d", o.p.who(), o.p.y);
    $display("T| o.q who=%s y=%0d", o.q.who(), o.q.y);
    b = o.p;
    $display("T| b who=%s", b.who());
  end
endmodule
"#;
    check(
        "t6",
        SRC,
        &[
            "T| v who=via_td/wrap_c/derived_c y=7 cast=1",
            "T| a who=w3/wrap_c/derived_c y=7",
            "T| ab who=w3/wrap_c/base_c cast=0",
            "T| tn a=class w3 #(class derived_c) ab=class w3 #(class base_c)",
            "T| o.p who=wrap_c/derived_c y=7",
            "T| o.q who=wrap_c/derived_c y=7",
            "T| b who=wrap_c/derived_c",
        ],
    );
}

/// A type-parameter base whose DEFAULT is itself a specialization
/// (`type BASE = comp_base#(byte)`): the unspecialized class binds `comp_base#(byte)`, not
/// `comp_base`'s own default, for statics, `$typename` and `$cast`.
#[test]
fn type_param_base_specialized_default() {
    const SRC: &str = r#"
class comp_base #(type T = int);
  T data;
  static int sc;
  function new(); sc++; endfunction
  virtual function string who(); return $sformatf("comp_base#(%s)", $typename(T)); endfunction
endclass
class wrap_d #(type BASE = comp_base#(byte)) extends BASE;
  virtual function string who(); return {"wrap_d/", super.who()}; endfunction
endclass
class user_d extends wrap_d #(comp_base#(byte));
endclass
class user_s extends wrap_d #(comp_base#(shortint));
endclass
module tb;
  wrap_d w; user_d u; user_s s; comp_base#(byte) cb;
  initial begin
    w = new(); u = new(); s = new();
    $display("T| %s", w.who());
    $display("T| %s", u.who());
    $display("T| %s", s.who());
    $display("T| sc %0d %0d %0d", comp_base#(byte)::sc, comp_base#(shortint)::sc, comp_base#()::sc);
    $display("T| cast %0d %0d", $cast(cb, w), $cast(cb, u));
    $display("T| tn %s", $typename(w));
  end
endmodule
"#;
    check(
        "t7",
        SRC,
        &[
            "T| wrap_d/comp_base#(byte)",
            "T| wrap_d/comp_base#(byte)",
            "T| wrap_d/comp_base#(shortint)",
            "T| sc 2 1 0",
            "T| cast 1 1",
            "T| tn class wrap_d #(class comp_base #(byte))",
        ],
    );
}

/// §6.20.3: any type-parameter default written as a specialization (`type B = box#(T)`,
/// `type D = box#(shortint)`) keeps its arguments, following earlier parameters.
#[test]
fn specialized_type_param_default_keeps_its_arguments() {
    const SRC: &str = r#"
class box #(type T = int); T v; endclass
class C #(type T = byte, type B = box#(T), type D = box#(shortint));
  function string s(); return {$typename(B), " / ", $typename(D)}; endfunction
endclass
module tb;
  C c; C#(int) ci;
  initial begin c = new(); ci = new(); $display("T| %s", c.s()); $display("T| %s", ci.s()); end
endmodule
"#;
    check(
        "t8",
        SRC,
        &[
            "T| class box #(byte) / class box #(shortint)",
            "T| class box #(int) / class box #(shortint)",
        ],
    );
}

/// `$typename(this)`, a local of the base type parameter, and `this_type` construction
/// inside a method of a non-default specialization.
#[test]
fn type_param_base_typename_this_and_this_type() {
    const SRC: &str = r#"
class base_c;
  virtual function string who(); return "base_c"; endfunction
endclass
class derived_c extends base_c;
  virtual function string who(); return "derived_c"; endfunction
endclass
class wrap_c #(type BASE = base_c) extends BASE;
  typedef wrap_c #(BASE) this_type;
  function string tn(); return $typename(this); endfunction
  function string bn(); BASE b; return $typename(b); endfunction
  function this_type cp(); this_type t; t = new(); return t; endfunction
endclass
module tb;
  wrap_c w; wrap_c #(derived_c) wd; wrap_c #(derived_c) wd2;
  initial begin
    w = new(); wd = new();
    $display("T| %s | %s", w.tn(), wd.tn());
    $display("T| %s | %s", w.bn(), wd.bn());
    wd2 = wd.cp();
    $display("T| cp %s %s", wd2.who(), wd2.tn());
  end
endmodule
"#;
    check(
        "t9",
        SRC,
        &[
            "T| class wrap_c #(class base_c) | class wrap_c #(class derived_c)",
            "T| class base_c | class derived_c",
            "T| cp derived_c class wrap_c #(class derived_c)",
        ],
    );
}

/// One class entry per bound base CLASS: the base's own arguments (`pbase#(8, byte)`) reach
/// it through the specialization, also two levels down (`wrap2_c #(pbase#(6))`) and from
/// a non-parameterized subclass, for value and type parameters and per-spec statics.
#[test]
fn type_param_base_projected_arguments() {
    const SRC: &str = r#"
class base_c;
  function new(); endfunction
  virtual function string who(); return "base_c"; endfunction
endclass
class pbase #(int W = 4, type T = int) extends base_c;
  T tv;
  static int cnt;
  function new(); super.new(); cnt++; endfunction
  function int w(); return W; endfunction
  virtual function string who(); return $sformatf("pbase%0d/%s", W, $typename(T)); endfunction
endclass
class wrap_c #(type BASE = base_c) extends BASE;
  virtual function string who(); return {"wrap_c/", super.who()}; endfunction
endclass
class wrap2_c #(type B = base_c) extends wrap_c #(B);
  virtual function string who(); return {"wrap2_c/", super.who()}; endfunction
endclass
class leaf8 extends wrap_c #(pbase#(8, byte));
endclass
module tb;
  leaf8 l; wrap2_c #(pbase#(6)) w6; wrap_c #(pbase#(7, shortint)) w7; wrap_c #(pbase) w4;
  pbase #(8, byte) pb8; base_c b;
  initial begin
    l = new(); w6 = new(); w7 = new(); w4 = new();
    $display("T| l %s %0d", l.who(), l.w());
    $display("T| w6 %s %0d", w6.who(), w6.w());
    $display("T| w7 %s %0d", w7.who(), w7.w());
    $display("T| w4 %s %0d", w4.who(), w4.w());
    $display("T| cnt %0d %0d %0d %0d", pbase#(8, byte)::cnt, pbase#(6)::cnt, pbase#(7, shortint)::cnt, pbase#()::cnt);
    $display("T| cast %0d", $cast(pb8, l));
    $display("T| tn %s | %s", $typename(w6), $typename(w7));
  end
endmodule
"#;
    check(
        "t10",
        SRC,
        &[
            "T| l wrap_c/pbase8/byte 8",
            "T| w6 wrap2_c/wrap_c/pbase6/int 6",
            "T| w7 wrap_c/pbase7/shortint 7",
            "T| w4 wrap_c/pbase4/int 4",
            "T| cnt 1 1 1 1",
            "T| cast 1",
            "T| tn class wrap2_c #(class pbase #(6, int)) | class wrap_c #(class pbase #(7, shortint))",
        ],
    );
}

/// `this_type` construction in a static method, a value parameter beside the type parameter,
/// a named `extends wrap_c #(.BASE(B))`, a typedef default, a specialization made inside
/// another parameterized class, a nested `wrap_c #(wrap_c #(derived_c))`, and
/// `randomize` with the bound base's constraints.
#[test]
fn type_param_base_nested_named_value_params_randomize() {
    const SRC: &str = r#"
class base_c;
  rand int r;
  int x = 5;
  function new(); endfunction
  virtual function string who(); return "base_c"; endfunction
endclass
class derived_c extends base_c;
  int y = 7;
  constraint c_y { r < 100; r > 50; }
  virtual function string who(); return "derived_c"; endfunction
endclass
typedef derived_c dalias;
class wrap_c #(type BASE = base_c) extends BASE;
  typedef wrap_c #(BASE) this_type;
  static int wcnt;
  function new(); super.new(); wcnt++; endfunction
  virtual function string who(); return {"wrap_c/", super.who()}; endfunction
  static function this_type make(); this_type t = new(); return t; endfunction
  static function int count(); return wcnt; endfunction
endclass
class wv #(type BASE = base_c, int N = 2) extends BASE;
  function int n(); return N; endfunction
  virtual function string who(); return $sformatf("wv%0d/%s", N, super.who()); endfunction
endclass
class wn #(type B = base_c) extends wrap_c #(.BASE(B));
  virtual function string who(); return {"wn/", super.who()}; endfunction
endclass
class wa #(type B = dalias) extends B;
  virtual function string who(); return {"wa/", super.who()}; endfunction
endclass
class holder #(type T = base_c);
  static function string mk(); wrap_c #(T) w = new(); return w.who(); endfunction
endclass
module tb;
  wrap_c #(derived_c) wd, wd2; wv #(derived_c, 5) v; wv v2; wn #(derived_c) n; wa a;
  wrap_c #(wrap_c #(derived_c)) ww; base_c b;
  initial begin
    wd = wrap_c#(derived_c)::make();
    $display("T| make who=%s y=%0d cnt=%0d %0d", wd.who(), wd.y, wrap_c#(derived_c)::count(), wrap_c#()::count());
    $display("T| tn=%s", $typename(wd));
    v = new(); v2 = new();
    $display("T| v who=%s n=%0d y=%0d v2 who=%s n=%0d", v.who(), v.n(), v.y, v2.who(), v2.n());
    n = new();
    $display("T| n who=%s y=%0d", n.who(), n.y);
    a = new();
    $display("T| a who=%s y=%0d", a.who(), a.y);
    $display("T| holder %s %s", holder#(derived_c)::mk(), holder#()::mk());
    ww = new();
    $display("T| ww who=%s y=%0d", ww.who(), ww.y);
    $display("T| tn ww=%s", $typename(ww));
    b = ww;
    $display("T| cast %0d %0d", $cast(wd2, b), $cast(wd2, ww));
    wd = new();
    void'(wd.randomize());
    $display("T| rand ok=%0d", wd.r > 50 && wd.r < 100);
    void'(wd.randomize() with { r == 77; });
    $display("T| rand with r=%0d", wd.r);
    $finish;
  end
endmodule
"#;
    check(
        "t5",
        SRC,
        &[
            "T| make who=wrap_c/derived_c y=7 cnt=1 0",
            "T| tn=class wrap_c #(class derived_c)",
            "T| v who=wv5/derived_c n=5 y=7 v2 who=wv2/base_c n=2",
            "T| n who=wn/wrap_c/derived_c y=7",
            "T| a who=wa/derived_c y=7",
            "T| holder wrap_c/derived_c wrap_c/base_c",
            "T| ww who=wrap_c/wrap_c/derived_c y=7",
            "T| tn ww=class wrap_c #(class wrap_c #(class derived_c))",
            "T| cast 1 1",
            "T| rand ok=1",
            "T| rand with r=77",
        ],
    );
}

/// §8.26.4: an interface class shall not extend a type parameter; the
/// reference rejects it at compile time.
#[test]
fn interface_class_extending_type_parameter_is_rejected() {
    let src = r#"interface class ic_base;
  pure virtual function int f();
endclass
interface class ic_w #(type B = ic_base) extends B;
  pure virtual function int g();
endclass
module tb;
endmodule
"#;
    let dir = std::env::temp_dir().join(format!("xezim_tpbase_ic_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("ic.sv");
    std::fs::write(&sv, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg("--compile")
        .arg("-s")
        .arg("tb")
        .arg(sv.to_str().unwrap())
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "expected a compile error:\n{all}");
    assert!(
        all.contains("shall not extend a type parameter"),
        "missing diagnostic:\n{all}"
    );
}
