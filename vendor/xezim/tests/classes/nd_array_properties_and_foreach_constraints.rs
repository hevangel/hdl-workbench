//! §7.4.2 / §18.5.8 — MULTI-dimensional fixed-array class properties, and
//! `foreach` constraints over them. Reference-validated.
//!
//! Three coupled defects — the first is what made the other two invisible:
//!
//! 1. **`obj.m[i][j]` never resolved to element storage.** Construction seeds
//!    a per-instance cell for every element of the shape, but no read or write
//!    path resolved the index chain: the property fell through to a packed
//!    bit-select of its unused scalar cell, so `[0][0]` read a stray bit and
//!    everything else read x — while the 1-D `obj.m[i]` beside it worked.
//! 2. **A `foreach (m[i,j])` constraint was dropped silently.** The solver arm
//!    consulted only the 1-D shape table and bound only the first variable, so
//!    it found no range and returned false; `randomize()` still returned 1.
//! 3. **N-D arrays were in NEITHER element pipeline** — not in the fixed-array
//!    pool pass (built from the 1-D table) and skipped by the collection
//!    pipeline — so nothing seeded them and nothing coordinated a baseline
//!    with the element solver.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Plain element reads and writes on a 2-D property, from outside and inside
/// the class.
#[test]
fn nd_property_elements_read_and_write() {
    let src = r#"
class C;
  bit [7:0] m[2][3];
  int seen;
  function void fill();  m[1][2] = 8'h5A; endfunction
  function void look();  seen = m[1][2];  endfunction
endclass
module tb;
  C c;
  int direct, inside_v, wrote, pre;
  initial begin
    c = new();
    pre = c.m[0][1];             // seeded, not x
    c.fill();
    direct = c.m[1][2];
    c.look(); inside_v = c.seen;
    c.m[0][1] = 8'h33;           // write from outside
    wrote = c.m[0][1];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "pre"), 0, "elements are seeded, not x");
    assert_eq!(u(&sim, "direct"), 0x5A, "written inside, read outside");
    assert_eq!(u(&sim, "inside_v"), 0x5A, "read inside");
    assert_eq!(u(&sim, "wrote"), 0x33, "written outside, read back");
}

/// The 2-D foreach constraint holds over repeated randomize calls, next to a
/// 1-D control.
#[test]
fn foreach_constraint_over_a_2d_rand_array() {
    let src = r#"
class RC;
  rand bit [3:0] m[2][3];
  constraint c_m { foreach (m[i,j]) m[i][j] inside {[1:5]}; }
endclass
class RC1;
  rand bit [3:0] m[3];
  constraint c_m { foreach (m[i]) m[i] inside {[1:5]}; }
endclass
module tb;
  RC rc; RC1 rc1;
  int ok_1d, ok_2d;
  initial begin
    rc1 = new(); ok_1d = 1;
    for (int i = 0; i < 20; i++) begin
      if (!rc1.randomize()) ok_1d = 0;
      foreach (rc1.m[p]) if (!(rc1.m[p] >= 1 && rc1.m[p] <= 5)) ok_1d = 0;
    end
    rc = new(); ok_2d = 1;
    for (int i = 0; i < 20; i++) begin
      if (!rc.randomize()) ok_2d = 0;
      foreach (rc.m[p,q]) if (!(rc.m[p][q] >= 1 && rc.m[p][q] <= 5)) ok_2d = 0;
    end
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "ok_1d"), 1, "the 1-D control still holds");
    assert_eq!(
        u(&sim, "ok_2d"),
        1,
        "every element of every draw is in range"
    );
}

/// An UNconstrained 2-D rand array must still be randomized (the pool pass now
/// owns it), not left at its seeded zeros.
#[test]
fn unconstrained_2d_rand_array_gets_randomized() {
    let src = r#"
class R;
  rand bit [7:0] m[2][3];
endclass
module tb;
  R r;
  int nonzero;
  initial begin
    r = new();
    nonzero = 0;
    for (int t = 0; t < 5; t++) begin
      void'(r.randomize());
      foreach (r.m[p,q]) if (r.m[p][q] != 0) nonzero++;
    end
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert!(
        u(&sim, "nonzero") > 0,
        "elements take random values across draws"
    );
}
