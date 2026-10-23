//! §8.9: a static class property read or written through an object handle
//! names the class's one shared cell. The handle's declared type decides
//! which class (and, §8.25, which specialization of a parameterized class),
//! so the access is legal through a null handle, and a derived object's
//! same-named instance property does not hide the base's static from a
//! base-typed handle.
//!
//! Before the fix such a read gave x (0 when assigned) unless the cell had
//! already been touched through `C::` (the handle path never seeded it),
//! and a null handle, a derived class's own static, and every
//! parameterized class's statics read x; writes through a handle to a
//! specialization's static were lost. Expected lines come from the
//! reference simulator.

const PFX: &str = "sth";
use std::process::Command;

/// Run `src` with top `top` and return its `T|` lines.
fn run(tag: &str, src: &str, top: &str) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("xezim_{}_{}_{}", PFX, tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("t.sv");
    std::fs::write(&sv, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg("--simulate")
        .arg("-s")
        .arg(top)
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

fn check(tag: &str, src: &str, top: &str, expected: &[&str]) {
    let got = run(tag, src, top);
    assert_eq!(
        got,
        expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "T| lines differ from the reference"
    );
}

/// Plain, null, derived and parameterized-class handles at module scope; reads,
/// writes, `++` and `+=` through a handle; `C::ds` as the control.
#[test]
fn static_read_write_through_handle_null_and_specialization() {
    const SRC: &str = r#"
class C;
  static int ds = 4;
  int inst = 1;
  function int get_ds(); return ds; endfunction
  function int get_this_ds(); return this.ds; endfunction
  function void set_ds(int v); this.ds = v; endfunction
endclass
class D extends C;
  static int dd = 9;
  function int sum(); return ds + dd + this.ds; endfunction
endclass
class P #(int W = 3);
  static int ps = W;
  static int cnt;
endclass
module top;
  C d, n;
  D dd;
  P#(5) p5; P#(6) p6; P p3;
  int v;
  initial begin
    d = new;
    $display("T| C::ds=%0d", C::ds);
    $display("T| d.ds=%0d", d.ds);
    v = d.ds; $display("T| v=d.ds=%0d", v);
    v = d.ds + 1; $display("T| v=d.ds+1=%0d", v);
    d.ds = 11; $display("T| after d.ds=11 C::ds=%0d d.ds=%0d", C::ds, d.ds);
    $display("T| null n.ds=%0d", n.ds);
    v = n.ds; $display("T| v=n.ds=%0d", v);
    n.ds = 12; $display("T| after n.ds=12 C::ds=%0d", C::ds);
    dd = new;
    $display("T| dd.ds=%0d dd.dd=%0d", dd.ds, dd.dd);
    dd.ds = 13; $display("T| after dd.ds=13 C::ds=%0d D::ds=%0d", C::ds, D::ds);
    $display("T| dd.sum=%0d", dd.sum());
    $display("T| get_ds=%0d get_this_ds=%0d", d.get_ds(), d.get_this_ds());
    d.set_ds(14); $display("T| after set_ds C::ds=%0d", C::ds);
    p5 = new; p6 = new;
    $display("T| p5.ps=%0d p6.ps=%0d p3.ps=%0d", p5.ps, p6.ps, p3.ps);
    p5.cnt = 1; p6.cnt = 2; p3.cnt = 3;
    $display("T| cnt %0d %0d %0d", P#(5)::cnt, P#(6)::cnt, P#()::cnt);
    $display("T| cnt via h %0d %0d %0d", p5.cnt, p6.cnt, p3.cnt);
    v = p6.ps; $display("T| v=p6.ps=%0d", v);
    p5.cnt++; $display("T| p5.cnt++ -> %0d", P#(5)::cnt);
    d.ds += 2; $display("T| d.ds+=2 -> %0d", C::ds);
  end
endmodule
"#;
    check(
        "static_read_write_through_handle_null_and_specialization",
        SRC,
        "top",
        &[
            "T| C::ds=4",
            "T| d.ds=4",
            "T| v=d.ds=4",
            "T| v=d.ds+1=5",
            "T| after d.ds=11 C::ds=11 d.ds=11",
            "T| null n.ds=11",
            "T| v=n.ds=11",
            "T| after n.ds=12 C::ds=12",
            "T| dd.ds=12 dd.dd=9",
            "T| after dd.ds=13 C::ds=13 D::ds=13",
            "T| dd.sum=35",
            "T| get_ds=13 get_this_ds=13",
            "T| after set_ds C::ds=14",
            "T| p5.ps=5 p6.ps=6 p3.ps=3",
            "T| cnt 1 2 3",
            "T| cnt via h 1 2 3",
            "T| v=p6.ps=6",
            "T| p5.cnt++ -> 2",
            "T| d.ds+=2 -> 16",
        ],
    );
}

/// Statics through formals, locals and properties inside a method, through
/// null handles of each specialization, a typedef of a specialization and a
/// subclass of one.
#[test]
fn static_through_handle_in_methods_and_specializations() {
    const SRC: &str = r#"
class P #(int W = 3, type T = int);
  static int cnt = W;
  static T tv;
  int inst = W;
endclass
class Q extends P#(9);
  static string nm = "q";
endclass
typedef P#(4) p4_t;
class C;
  static int ds = 4;
  static string ss = "c";

  C other;
  P#(5) pp;
  function void m(C arg, P#(6) parg);
    C loc;
    P#(7) ploc;
    $display("T| m: arg.ds=%0d loc.ds=%0d other.ds=%0d this.other.ds=%0d", arg.ds, loc.ds, other.ds, this.other.ds);
    $display("T| m: pp.cnt=%0d parg.cnt=%0d ploc.cnt=%0d", pp.cnt, parg.cnt, ploc.cnt);
    loc.ds = 21; $display("T| m: loc.ds=21 -> %0d", C::ds);
    ploc.cnt = 70; $display("T| m: ploc.cnt=70 -> %0d %0d", P#(7)::cnt, ploc.cnt);
    parg.cnt += 1; $display("T| m: parg.cnt+=1 -> %0d", P#(6)::cnt);
    other.ss = "x"; $display("T| m: ss=%s %s", C::ss, loc.ss);

  endfunction
endclass
module top;
  C c, n;
  P#(5) p5n;
  P#(6) p6;
  P#(5, byte) p5b;
  Q q, qn;
  p4_t t4;
  int v;
  string s;
  initial begin
    c = new;
    c.other = new;
    $display("T| p5n.cnt=%0d p6.cnt=%0d p5b.cnt=%0d", p5n.cnt, p6.cnt, p5b.cnt);
    p5b.cnt = 55; $display("T| P#(5,byte)=%0d P#(5)=%0d", P#(5,byte)::cnt, P#(5)::cnt);
    $display("T| qn.cnt=%0d qn.nm=%s", qn.cnt, qn.nm);
    q = new; q.cnt = 99; $display("T| P#(9)::cnt=%0d qn.cnt=%0d", P#(9)::cnt, qn.cnt);
    $display("T| t4.cnt=%0d", t4.cnt);
    t4.cnt = 44; $display("T| P#(4)::cnt=%0d", P#(4)::cnt);
    v = c.other.ds; $display("T| c.other.ds=%0d", v);
    c.m(null, null);
    s = n.ss; $display("T| n.ss=%s", s);

    $display("T| C::ds=%0d", C::ds);
  end
endmodule
"#;
    check(
        "static_through_handle_in_methods_and_specializations",
        SRC,
        "top",
        &[
            "T| p5n.cnt=5 p6.cnt=6 p5b.cnt=5",
            "T| P#(5,byte)=55 P#(5)=5",
            "T| qn.cnt=9 qn.nm=q",
            "T| P#(9)::cnt=99 qn.cnt=99",
            "T| t4.cnt=4",
            "T| P#(4)::cnt=44",
            "T| c.other.ds=4",
            "T| m: arg.ds=4 loc.ds=4 other.ds=4 this.other.ds=4",
            "T| m: pp.cnt=5 parg.cnt=6 ploc.cnt=7",
            "T| m: loc.ds=21 -> 21",
            "T| m: ploc.cnt=70 -> 70 70",
            "T| m: parg.cnt+=1 -> 7",
            "T| m: ss=x x",
            "T| n.ss=x",
            "T| C::ds=21",
        ],
    );
}

/// A property declared through a typedef of a specialization, a chain ending
/// in a null handle, and a base-typed handle to a derived object whose own
/// instance property shares the static's name.
#[test]
fn static_through_typedef_property_and_chain() {
    const SRC: &str = r#"
class P #(int W = 3);
  static int cnt = W;
endclass
typedef P#(4) p4_t;
class C;
  static int ds = 4;
  C other;
  p4_t tp;
  P#(8) p8;
  function void m();
    $display("T| m tp.cnt=%0d p8.cnt=%0d", tp.cnt, p8.cnt);
    tp.cnt = 40; p8.cnt = 80;
    $display("T| m after %0d %0d", P#(4)::cnt, P#(8)::cnt);
    other.other.ds = 7;
    $display("T| m chain %0d", C::ds);
  endfunction
endclass
class D extends C;
  int ds = 100;
endclass
module top;
  C c; D d; C cd;
  initial begin
    c = new; c.other = new;
    c.m();
    c.other.ds = 9; $display("T| c.other.ds=9 -> %0d", C::ds);
    d = new; cd = d;
    $display("T| d.ds=%0d cd.ds=%0d C::ds=%0d", d.ds, cd.ds, C::ds);
    cd.ds = 11; $display("T| cd.ds=11 -> C::ds=%0d d.ds=%0d", C::ds, d.ds);
    d.ds = 12; $display("T| d.ds=12 -> C::ds=%0d d.ds=%0d", C::ds, d.ds);
  end
endmodule
"#;
    check(
        "static_through_typedef_property_and_chain",
        SRC,
        "top",
        &[
            "T| m tp.cnt=4 p8.cnt=8",
            "T| m after 40 80",
            "T| m chain 7",
            "T| c.other.ds=9 -> 9",
            "T| d.ds=100 cd.ds=9 C::ds=9",
            "T| cd.ds=11 -> C::ds=11 d.ds=100",
            "T| d.ds=12 -> C::ds=11 d.ds=12",
        ],
    );
}

/// The declared type of the handle decides: `C cd = d;` reaches `C::ds`, `d.ds`
/// the instance property of `D`, inside methods too; chains `a.b.s` and chains
/// ending in a null property.
#[test]
fn static_shadowed_by_derived_instance_property() {
    const SRC: &str = r#"
class C;
  static int ds = 4;
  C other;
  function void rd(C h); $display("T| rd h.ds=%0d", h.ds); endfunction
endclass
class D extends C;
  int ds = 100;
  function void rdd(C h, D e); $display("T| rdd h.ds=%0d e.ds=%0d", h.ds, e.ds); h.ds = 5; e.ds = 101; endfunction
endclass
class W;
  C c;
  function new(); c = new; c.other = new; endfunction
  function void go(); c.other.ds = 7; $display("T| W chain -> %0d", C::ds); c.other.other.ds = 8; $display("T| W null chain -> %0d", C::ds); endfunction
endclass
module top;
  D d; C cd; W w; int v;
  initial begin
    d = new; cd = d;
    v = cd.ds; $display("T| v=%0d", v);
    $display("T| cd.ds=%0d d.ds=%0d", cd.ds, d.ds);
    cd.rd(d); d.rdd(d, d);
    $display("T| C::ds=%0d d.ds=%0d", C::ds, d.ds);
    w = new; w.go();
    w.c.other.ds = 9; $display("T| top chain -> %0d %0d %0d", C::ds, w.c.other.ds, w.c.other.other.ds);
  end
endmodule
"#;
    check(
        "static_shadowed_by_derived_instance_property",
        SRC,
        "top",
        &[
            "T| v=4",
            "T| cd.ds=4 d.ds=100",
            "T| rd h.ds=4",
            "T| rdd h.ds=4 e.ds=100",
            "T| C::ds=5 d.ds=101",
            "T| W chain -> 7",
            "T| W null chain -> 8",
            "T| top chain -> 9 9 9",
        ],
    );
}

/// A class whose base is a type parameter: the statics of its own and of the
/// bound base, live and null, per specialization.
#[test]
fn static_through_handle_type_param_base() {
    const SRC: &str = r#"
class base_c; static int bs = 2; endclass
class derived_c extends base_c; static int ds = 3; endclass
class wrap_c #(type BASE = base_c) extends BASE;
  static int ws = 1;
endclass
typedef wrap_c#(derived_c) wd_t;
module tb;
  wd_t wt, wn; wrap_c w0, wz; wrap_c#(derived_c) wx;
  initial begin
    wt = new; w0 = new;
    $display("T| live %0d %0d %0d %0d", wt.ds, wt.bs, wt.ws, w0.bs);
    $display("T| null %0d %0d %0d %0d %0d", wn.ds, wn.bs, wn.ws, wz.bs, wx.ds);
    wn.ds = 30; wx.ws = 10; wz.ws = 11;
    $display("T| after %0d %0d %0d %0d", derived_c::ds, wrap_c#(derived_c)::ws, wrap_c#()::ws, wt.ws);
  end
endmodule
"#;
    check(
        "static_through_handle_type_param_base",
        SRC,
        "tb",
        &[
            "T| live 3 2 1 2",
            "T| null 3 2 1 2 3",
            "T| after 30 10 11 10",
        ],
    );
}

/// The first access to a static is through a handle (the cell had not been
/// seeded by a `C::` access yet).
#[test]
fn static_first_access_through_handle() {
    const SRC: &str = r#"
class base_c; static int bs = 4; endclass
class derived_c extends base_c; static int ds = 3; int v = 1; endclass
module tb;
  derived_c d; int r1, r2;
  initial begin
    derived_c l;
    d = new(); l = d;
    r1 = d.ds; r2 = l.ds;
    $display("T| %0d %0d %0d %0d %0d", d.ds, l.ds, r1, r2, d.v);
    $display("T| %0d", d.ds + 0);
  end
endmodule
"#;
    check(
        "static_first_access_through_handle",
        SRC,
        "tb",
        &["T| 3 3 3 3 1", "T| 3"],
    );
}

/// Inside a parameterized class, statics through `this_type` handles (live,
/// `m_inst`, null) and through a property typed with a specialization each
/// reach the running specialization's cell.
#[test]
fn static_through_this_type_per_specialization() {
    const SRC: &str = r#"
class cb #(type T = int);
  typedef cb#(T) this_type;
  static this_type m_inst;
  static int cnt;
  T v;
  static function this_type get();
    if (m_inst == null) m_inst = new;
    return m_inst;
  endfunction
  function void bump();
    this_type h = this;
    this_type n;
    h.cnt++;
    m_inst.cnt += 10;
    n.cnt += 100;
    $display("T| bump %0d %0d %0d", h.cnt, m_inst.cnt, n.cnt);
  endfunction
endclass
class user;
  cb#(byte) p;
  function void go(); p = cb#(byte)::get(); p.cnt += 1000; endfunction
endclass
module top;
  user u;
  initial begin
    cb#(int)::get().bump();
    cb#(byte)::get().bump();
    cb#(byte)::get().bump();
    u = new; u.go();
    $display("T| %0d %0d", cb#(int)::cnt, cb#(byte)::cnt);
  end
endmodule
"#;
    check(
        "static_through_this_type_per_specialization",
        SRC,
        "top",
        &[
            "T| bump 111 111 111",
            "T| bump 111 111 111",
            "T| bump 222 222 222",
            "T| 111 1222",
        ],
    );
}
