//! §8.25 / §8.16: `$cast` to a destination whose type is a specialization of
//! a parameterized class succeeds only when the source object is (or
//! derives from) that same specialization. The check used to read only a
//! procedural local's declared `#(...)` arguments, so a destination declared
//! at module scope, as a class property, as a formal, through a typedef or
//! without arguments (the default specialization) passed every cast of the
//! right class. Expected lines come from the reference simulator.

const PFX: &str = "cds";
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

/// `leaf extends pbase#(7, shortint)` cast to `pbase#(8, byte)` and friends
/// declared at module scope, through a typedef, unspecialized, as a property, a
/// local, an input formal and an output formal.
#[test]
fn cast_to_ancestor_specialization_by_declaration_scope() {
    const SRC: &str = r#"
class pbase #(int N = 1, type T = int);
  T v;
  function int n(); return N; endfunction
endclass
class leaf extends pbase#(7, shortint);
endclass
typedef pbase#(8, byte) pb8_t;
typedef pbase#(7, shortint) pb7_t;
class holder;
  pbase#(8, byte) hp;
  pbase#(7, shortint) hq;
endclass
module top;
  pbase#(8, byte) mx;
  pbase#(7, shortint) my;
  pbase#(7, int) mz;
  pbase#(8, shortint) mw;
  pb8_t mt;
  pb7_t mu;
  pbase mdef;
  leaf obj;
  holder h;
  int r;
  function automatic int f_cast(pbase#(8, byte) fx, leaf o);
    return $cast(fx, o);
  endfunction
  task automatic t_cast(output pbase#(8, byte) fx, input leaf o, output int rr);
    rr = $cast(fx, o);
  endtask
  function automatic int f_cast7(pbase#(7, shortint) fx, leaf o);
    return $cast(fx, o);
  endfunction
  initial begin
    obj = new;
    h = new;
    r = $cast(mx, obj); $display("T| mod 8,byte r=%0d null=%0d", r, mx == null);
    r = $cast(my, obj); $display("T| mod 7,shortint r=%0d null=%0d", r, my == null);
    r = $cast(mz, obj); $display("T| mod 7,int r=%0d", r);
    r = $cast(mw, obj); $display("T| mod 8,shortint r=%0d", r);
    r = $cast(mt, obj); $display("T| typedef 8,byte r=%0d", r);
    r = $cast(mu, obj); $display("T| typedef 7,shortint r=%0d", r);
    r = $cast(mdef, obj); $display("T| default r=%0d", r);
    r = $cast(h.hp, obj); $display("T| prop 8,byte r=%0d", r);
    r = $cast(h.hq, obj); $display("T| prop 7,shortint r=%0d", r);
    begin
      pbase#(8, byte) lx;
      pbase#(7, shortint) ly;
      r = $cast(lx, obj); $display("T| local 8,byte r=%0d", r);
      r = $cast(ly, obj); $display("T| local 7,shortint r=%0d", r);
    end
    r = f_cast(null, obj); $display("T| formal 8,byte r=%0d", r);
    r = f_cast7(null, obj); $display("T| formal 7,shortint r=%0d", r);
    begin pbase#(8, byte) ox; t_cast(ox, obj, r); $display("T| output formal 8,byte r=%0d", r); end
  end
endmodule
"#;
    check(
        "cast_to_ancestor_specialization_by_declaration_scope",
        SRC,
        "top",
        &[
            "T| mod 8,byte r=0 null=1",
            "T| mod 7,shortint r=1 null=0",
            "T| mod 7,int r=0",
            "T| mod 8,shortint r=0",
            "T| typedef 8,byte r=0",
            "T| typedef 7,shortint r=1",
            "T| default r=0",
            "T| prop 8,byte r=0",
            "T| prop 7,shortint r=1",
            "T| local 8,byte r=0",
            "T| local 7,shortint r=1",
            "T| formal 8,byte r=0",
            "T| formal 7,shortint r=1",
            "T| output formal 8,byte r=0",
        ],
    );
}

/// Equal specializations spelled differently (named and expression arguments,
/// typedef chains) pass; a parameterized derived class, the default
/// specialization, type-only and string parameters, and bare property names
/// inside a method.
#[test]
fn cast_specialization_matches_and_mismatches() {
    const SRC: &str = r#"
class pbase #(int N = 1, type T = int);
  T v;
endclass
class pd #(int M = 2) extends pbase#(M, byte);
endclass
class tc #(type T = int);
endclass
class sc #(string S = "a");
endclass
class leaf extends pbase#(7, shortint);
endclass
typedef pbase#(7, shortint) p7_t;
typedef p7_t p7b_t;
class holder;
  pbase#(8, byte) hp;
  pbase#(7, shortint) hq;
  p7_t ht;
  function void m(leaf o, pd#(8) o8);
    int r;
    r = $cast(hp, o); $display("T| m bare hp r=%0d", r);
    r = $cast(hq, o); $display("T| m bare hq r=%0d", r);
    r = $cast(this.hq, o); $display("T| m this.hq r=%0d", r);
    r = $cast(ht, o); $display("T| m ht r=%0d", r);
    r = $cast(hp, o8); $display("T| m hp<-pd8 r=%0d", r);
  endfunction
endclass
module top;
  pbase#(7, shortint) a1;
  pbase#(.N(7), .T(shortint)) a2;
  pbase#(3+4, shortint) a3;
  p7b_t a4;
  pbase#(8, byte) b8;
  pbase#(2, byte) b2;
  pbase d0;
  pbase#(1, int) d1;
  tc#(byte) tb; tc#(int) ti; tc td;
  sc#("b") sb; sc#("a") sa; sc sd;
  leaf obj; pd#(8) o8; pd o2; pbase po; pbase#(8, byte) p8;
  tc#(byte) otb; sc#("b") osb;
  holder h;
  initial begin
    obj = new; o8 = new; o2 = new; po = new; p8 = new; otb = new; osb = new; h = new;
    $display("T| a1 %0d a2 %0d a3 %0d a4 %0d", $cast(a1, obj), $cast(a2, obj), $cast(a3, obj), $cast(a4, obj));
    $display("T| pd8->b8 %0d b2 %0d", $cast(b8, o8), $cast(b2, o8));
    $display("T| pd->b8 %0d b2 %0d", $cast(b8, o2), $cast(b2, o2));
    $display("T| po->d0 %0d d1 %0d b8 %0d", $cast(d0, po), $cast(d1, po), $cast(b8, po));
    $display("T| p8->b8 %0d a1 %0d d0 %0d", $cast(b8, p8), $cast(a1, p8), $cast(d0, p8));
    $display("T| tc tb %0d ti %0d td %0d", $cast(tb, otb), $cast(ti, otb), $cast(td, otb));
    $display("T| sc sb %0d sa %0d sd %0d", $cast(sb, osb), $cast(sa, osb), $cast(sd, osb));
    h.m(obj, o8);
    if (!$cast(a1, obj)) $display("T| task form fail"); else $display("T| task form ok");
  end
endmodule
"#;
    check(
        "cast_specialization_matches_and_mismatches",
        SRC,
        "top",
        &[
            "T| a1 1 a2 1 a3 1 a4 1",
            "T| pd8->b8 1 b2 0",
            "T| pd->b8 0 b2 1",
            "T| po->d0 1 d1 1 b8 0",
            "T| p8->b8 1 a1 0 d0 0",
            "T| tc tb 1 ti 0 td 0",
            "T| sc sb 1 sa 0 sd 0",
            "T| m bare hp r=0",
            "T| m bare hq r=1",
            "T| m this.hq r=1",
            "T| m ht r=1",
            "T| m hp<-pd8 r=1",
            "T| task form ok",
        ],
    );
}

/// `this_type` (a class-local typedef of the running specialization) as a
/// local and as a property, read inside the class and from module scope
/// through another object.
#[test]
fn cast_to_this_type_inside_specialization() {
    const SRC: &str = r#"
class base; endclass
class cb #(type T = int, int N = 1) extends base;
  typedef cb#(T, N) this_type;
  this_type peer;
  function int cp(base o);
    this_type t;
    return $cast(t, o);
  endfunction
  function int cpp(base o);
    return $cast(peer, o);
  endfunction
endclass
module top;
  cb#(int) ci; cb#(byte) cbt; cb#(byte, 2) cb2; base b;
  initial begin
    ci = new; cbt = new; cb2 = new;
    b = cbt;
    $display("T| %0d %0d %0d", ci.cp(b), cbt.cp(b), cb2.cp(b));
    $display("T| %0d %0d %0d", ci.cpp(b), cbt.cpp(b), cb2.cpp(b));
    b = cb2;
    $display("T| %0d %0d %0d", ci.cp(b), cbt.cp(b), cb2.cp(b));
    $display("T| %0d", $cast(cbt.peer, b));
    $display("T| %0d", $cast(cb2.peer, b));
  end
endmodule
"#;
    check(
        "cast_to_this_type_inside_specialization",
        SRC,
        "top",
        &["T| 0 1 0", "T| 0 1 0", "T| 0 0 1", "T| 0", "T| 1"],
    );
}
