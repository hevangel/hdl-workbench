//! §18.11 — `obj.randomize(a, b)` makes exactly the named members random for
//! that call, whether or not they are declared `rand` (and regardless of
//! rand_mode); every other member is state. The member list only selected
//! among the declared rand members, so naming a plain member left it
//! unchanged, and a constraint on it failed the call (`randomize(r, s)` with
//! `s` non-rand). Unlisted rand ARRAYS were still redrawn. Every expectation
//! was cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class inner_c;
  rand bit [7:0] p;
  bit [7:0] q;
  constraint ci { p inside {[1:9]}; }
endclass
class ca;
  rand byte x, y;
  byte v, w;
  rand bit [7:0] off;
  rand bit [7:0] a[4];
  rand bit [7:0] d[];
  bit [7:0] na[4];
  bit [7:0] nd[];
  typedef enum bit [1:0] {E0, E1, E2} e_t;
  e_t e;
  int signed si;
  rand inner_c in;
  constraint c1 { x < v && y > w; }
  constraint c2 { foreach (a[i]) a[i] < 100; foreach (na[i]) na[i] > 200; }
  constraint c3 { e != E0; si < -5; }
  function new(); in = new; d = new[3]; nd = new[3]; endfunction
endclass
class cb;
  rand bit [7:0] r;
  bit [7:0] s;
  constraint c { s inside {[10:20]}; r == s + 1; }
endclass
module top;
  initial begin
    automatic ca o = new;
    automatic cb b = new;
    automatic int ok, bad, vch, wch, xch, ych, ach, dch, nach, ndch, ech, sich, och, pch, qch, viol;
    automatic byte v0, w0, x0, y0;
    automatic bit [7:0] a0[4], d0[], na0[4], nd0[], o0, p0, q0;
    o.x = 0; o.y = 0; o.v = 50; o.w = -50; o.e = ca::E1; o.si = -10;
    o.na = '{255, 255, 255, 255};
    for (int i = 0; i < 100; i++) begin
      v0 = o.v; w0 = o.w; x0 = o.x; y0 = o.y; a0 = o.a; d0 = o.d;
      if (!o.randomize(v, w)) bad++; else ok++;
      if (o.v != v0) vch++; if (o.w != w0) wch++; if (o.x != x0) xch++; if (o.y != y0) ych++;
      if (o.a != a0) ach++; if (o.d != d0) dch++;
      if (!(o.x < o.v && o.y > o.w)) viol++;
    end
    $display("L1 ok=%0d bad=%0d vch=%0d wch=%0d xch=%0d ych=%0d ach=%0d dch=%0d viol=%0d",
             ok, bad, vch, wch, xch, ych, ach, dch, viol);
    ok = 0; bad = 0; viol = 0; ach = 0;
    for (int i = 0; i < 100; i++) begin
      na0 = o.na; nd0 = o.nd; a0 = o.a;
      if (!o.randomize(na, nd, e, si)) bad++; else ok++;
      if (o.na != na0) nach++; if (o.nd != nd0) ndch++; if (o.a != a0) ach++;
      if (o.e != ca::E1) ech++; if (o.si != -10) sich++;
      foreach (o.na[k]) if (o.na[k] <= 200) viol++;
      if (o.e == ca::E0 || o.si >= -5) viol++;
    end
    $display("L2 ok=%0d bad=%0d nach=%0d ndch=%0d ach=%0d emoved=%0d simoved=%0d viol=%0d",
             ok, bad, nach, ndch, ach, ech > 0, sich, viol);
    ok = 0; bad = 0; viol = 0;
    for (int i = 0; i < 100; i++) begin
      if (!b.randomize(r, s)) bad++;
      else if (b.s >= 10 && b.s <= 20 && b.r == b.s + 1) ok++;
      else viol++;
    end
    $display("L3 ok=%0d bad=%0d viol=%0d", ok, bad, viol);
    b.s = 3;
    $display("L4 state=%0d listed=%0d", b.randomize(), b.randomize(s));
    ok = 0; bad = 0;
    for (int i = 0; i < 50; i++) begin
      v0 = o.v;
      if (!o.randomize(v) with { v > 100; }) bad++; else if (o.v > 100) ok++;
      if (o.v != v0) vch++;
    end
    $display("L5 ok=%0d bad=%0d", ok, bad);
    ok = 0; bad = 0;
    o.off.rand_mode(0);
    for (int i = 0; i < 50; i++) begin
      o0 = o.off;
      if (!o.randomize(off)) bad++; else ok++;
      if (o.off != o0) och++;
    end
    $display("L6 ok=%0d bad=%0d moved=%0d", ok, bad, och > 40);
    ok = 0; bad = 0; viol = 0;
    for (int i = 0; i < 50; i++) begin
      p0 = o.in.p; q0 = o.in.q;
      if (!o.randomize(in)) bad++; else ok++;
      if (o.in.p != p0) pch++; if (o.in.q != q0) qch++;
      if (o.in.p < 1 || o.in.p > 9) viol++;
    end
    $display("L7 ok=%0d bad=%0d pmoved=%0d qch=%0d viol=%0d", ok, bad, pch > 30, qch, viol);
  end
endmodule
"#;

#[test]
fn listed_members_are_random_for_the_call() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with('L'))
        .collect();
    let line = |p: &str| {
        got.iter()
            .find(|l| l.starts_with(p))
            .unwrap_or_else(|| panic!("missing {p}: {got:?}"))
            .to_string()
    };
    let num = |l: &str, k: &str| -> i64 {
        l.split_whitespace()
            .find_map(|t| t.strip_prefix(k))
            .unwrap_or_else(|| panic!("{k} in {l}"))
            .parse()
            .unwrap()
    };
    // Non-rand `v`, `w` listed: both redrawn every call (a repeat of the same
    // byte is rare), the unlisted rand members and arrays keep their values.
    let l1 = line("L1 ");
    assert_eq!(num(&l1, "ok="), 100, "{l1}");
    assert!(num(&l1, "vch=") >= 90 && num(&l1, "wch=") >= 90, "{l1}");
    for k in ["xch=", "ych=", "ach=", "dch=", "viol="] {
        assert_eq!(num(&l1, k), 0, "{k} {l1}");
    }
    // Non-rand arrays, an enum and a signed int listed.
    let l2 = line("L2 ");
    assert_eq!(num(&l2, "ok="), 100, "{l2}");
    assert!(num(&l2, "nach=") >= 90 && num(&l2, "ndch=") >= 90, "{l2}");
    assert_eq!(num(&l2, "ach="), 0, "{l2}");
    assert_eq!(num(&l2, "emoved="), 1, "{l2}");
    assert!(num(&l2, "simoved=") >= 90, "{l2}");
    assert_eq!(num(&l2, "viol="), 0, "{l2}");
    assert_eq!(line("L3 "), "L3 ok=100 bad=0 viol=0");
    // `s` as state violates `s inside {[10:20]}`; listed, it is solved.
    assert_eq!(line("L4 "), "L4 state=0 listed=1");
    assert_eq!(line("L5 "), "L5 ok=50 bad=0");
    // A listed member is random even with rand_mode(0).
    assert_eq!(line("L6 "), "L6 ok=50 bad=0 moved=1");
    // A listed rand handle randomizes the whole inner object.
    assert_eq!(line("L7 "), "L7 ok=50 bad=0 pmoved=1 qch=0 viol=0");
}
