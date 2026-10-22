//! §8.10 — a property redeclared in a derived class is a SEPARATE variable.
//! Reference-validated.
//!
//! Instance storage was keyed by bare property name, so every declaration of
//! `s` in an inheritance chain collapsed into one slot — the seeding loop runs
//! root-to-leaf, so the derived's declaration silently overwrote the base's.
//! A base method's write became visible to the derived and vice versa, and
//! the corruption crossed TYPES: a base `string n` overwritten by a derived
//! `int n = 42` read back as the string "*" (character 42). Nothing looked
//! wrong at either write site.
//!
//! Now the leaf-most declarer keeps the bare key and every other declarer
//! stores under `"<Class>::<name>"`; a reference resolves through the
//! EXECUTING method's declaring class, `super.s` starts one class higher, and
//! external `obj.s` stays on the bare key (the leaf's copy, per §8.10).
//! Unshadowed names never leave the bare-name fast path.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The full matrix: base/derived methods, super, and external access, reads
/// and writes.
#[test]
fn base_and_derived_copies_are_distinct() {
    let src = r#"
class B;
  int s;
  function new(); s = 100; endfunction
  function int getb(); return s; endfunction
  function void setb(int v); s = v; endfunction
endclass
class D extends B;
  int s;
  function new(); super.new(); s = 200; endfunction
  function int getd(); return s; endfunction
  function int getsup(); return super.s; endfunction
  function void setsup(int v); super.s = v; endfunction
endclass
module tb;
  D d;
  int p1, p2, p3, p4, p5, p6, p7, p8, p9;
  initial begin
    d = new();
    p1 = d.getb();      // base method reads ITS copy
    p2 = d.getd();      // derived method reads ITS copy
    p3 = d.getsup();    // super.s = the base's copy
    p4 = d.s;           // external access = the leaf's copy
    d.setb(55);         // base method writes the base's copy...
    p5 = d.getb();
    p6 = d.getd();      // ...without touching the derived's
    p7 = d.s;
    d.s = 77;           // external write lands on the leaf's copy...
    p8 = d.getb();      // ...without touching the base's
    p9 = d.getd();
    d.setsup(31);       // super.s write reaches the base's copy
    p1 = d.getb();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "p1"), 31, "super.s write reaches the base's copy");
    assert_eq!(
        u(&sim, "p2"),
        200,
        "the derived keeps its constructor value"
    );
    assert_eq!(u(&sim, "p3"), 100, "super.s reads the base's copy");
    assert_eq!(u(&sim, "p4"), 200, "external access sees the leaf's copy");
    assert_eq!(
        u(&sim, "p5"),
        55,
        "base method write visible to base method"
    );
    assert_eq!(u(&sim, "p6"), 200, "and invisible to the derived");
    assert_eq!(u(&sim, "p7"), 200, "and invisible externally");
    assert_eq!(u(&sim, "p8"), 55, "external write invisible to the base");
    assert_eq!(u(&sim, "p9"), 77, "external write visible to the derived");
}

/// The cross-TYPE case: a base string must survive a derived int of the same
/// name.
#[test]
fn shadowing_does_not_cross_types() {
    let src = r#"
class B;
  bit [3:0] s;
  string n;
  function new(); s = 4'hF; n = "base"; endfunction
  function int slen(); return n.len(); endfunction
  function int sval(); return s; endfunction
endclass
class D extends B;
  bit [15:0] s;
  int n;
  function new(); super.new(); s = 16'hBEEF; n = 42; endfunction
endclass
module tb;
  D d;
  int b_len, b_s, d_n, d_s;
  initial begin
    d = new();
    b_len = d.slen();   // the base's string is intact
    b_s   = d.sval();   // the base's 4-bit value, not beef truncated
    d_n   = d.n;
    d_s   = d.s;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "b_len"),
        4,
        "the base's string survives the derived int"
    );
    assert_eq!(u(&sim, "b_s"), 0xF, "the base's width survives too");
    assert_eq!(u(&sim, "d_n"), 42);
    assert_eq!(u(&sim, "d_s"), 0xBEEF);
}

/// Three levels: each declarer gets its own copy; `super` from the middle
/// reaches the root's.
#[test]
fn three_level_shadowing() {
    let src = r#"
class A;
  int s;
  function new(); s = 1; endfunction
  function int geta(); return s; endfunction
endclass
class B extends A;
  int s;
  function new(); super.new(); s = 2; endfunction
  function int getb(); return s; endfunction
  function int getbsup(); return super.s; endfunction
endclass
class C extends B;
  int s;
  function new(); super.new(); s = 3; endfunction
  function int getc(); return s; endfunction
  function int getcsup(); return super.s; endfunction
endclass
module tb;
  C c;
  int va, vb, vc, vbs, vcs, ext;
  initial begin
    c = new();
    va = c.geta(); vb = c.getb(); vc = c.getc();
    vbs = c.getbsup();   // B's super.s = A's copy
    vcs = c.getcsup();   // C's super.s = B's copy
    ext = c.s;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!((u(&sim, "va"), u(&sim, "vb"), u(&sim, "vc")), (1, 2, 3));
    assert_eq!(u(&sim, "vbs"), 1, "middle super reaches the root");
    assert_eq!(u(&sim, "vcs"), 2, "leaf super reaches the middle");
    assert_eq!(u(&sim, "ext"), 3, "external = leaf");
}

/// An UNSHADOWED inherited property is one variable seen by everyone —
/// the fix must not split it.
#[test]
fn unshadowed_properties_stay_shared() {
    let src = r#"
class B;
  int t;
  function void setb(int v); t = v; endfunction
endclass
class D extends B;
  function int getd(); return t; endfunction
endclass
module tb;
  D d;
  int r1, r2;
  initial begin
    d = new();
    d.setb(9);
    r1 = d.getd();
    d.t = 11;
    r2 = d.getd();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r1"), 9, "one variable across the chain");
    assert_eq!(u(&sim, "r2"), 11);
}
