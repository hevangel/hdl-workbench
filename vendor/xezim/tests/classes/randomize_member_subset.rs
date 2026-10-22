//! §18.11 — `obj.randomize(<member subset>)`. Reference-validated.
//!
//! The arguments name MEMBERS of the object's class, not identifiers of the
//! calling scope. Elaboration validated them as ordinary expressions, so a
//! perfectly legal `d.randomize(a)` was rejected outright with "Undeclared
//! identifier 'a'" and the whole simulation aborted before producing output.
//!
//! Removing that rejection is only half of it: the listed members are the ONLY
//! ones solved, and every other rand member must keep its current value and
//! act as state. Randomizing everything would have satisfied the compile but
//! silently clobbered the caller's setup.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The named member is randomized under its constraint; the unnamed one is
/// left exactly as the caller set it.
#[test]
fn randomize_subset_leaves_other_members_alone() {
    let src = r#"
class B;
  rand bit [7:0] a;
  rand bit [7:0] b;
  constraint c_a { a inside {[10:20]}; }
  constraint c_b { b < 100; }
endclass
module tb;
  B d;
  int ok;
  initial begin
    ok = 1;
    d = new();
    d.b = 8'd77;
    for (int i = 0; i < 20; i++) begin
      if (!d.randomize(a)) ok = 0;
      if (d.b != 77) ok = 0;                    // untouched, even though rand
      if (!(d.a >= 10 && d.a <= 20)) ok = 0;    // still constrained
    end
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "ok"), 1);
}

/// Naming several members; the unlisted one is STATE — kept when it satisfies
/// its constraint, and making the call FAIL when it does not (both
/// reference-verified).
#[test]
fn unlisted_members_are_state() {
    let src = r#"
class B;
  rand bit [7:0] a, b, c;
  constraint lim { a < 50; b < 50; c < 50; }
endclass
module tb;
  B d;
  int ok_pair, c_kept, ok_all, unsat;
  initial begin
    ok_pair = 1; ok_all = 1;
    d = new();
    d.c = 8'd30;                     // satisfies c < 50, so it can be state
    for (int i = 0; i < 20; i++) begin
      if (!d.randomize(a, b)) ok_pair = 0;
      if (d.c != 30) ok_pair = 0;
      if (!(d.a < 50 && d.b < 50)) ok_pair = 0;
    end
    c_kept = d.c;
    for (int i = 0; i < 20; i++) begin
      if (!d.randomize()) ok_all = 0;   // whole object: c is solved too
      if (!(d.c < 50)) ok_all = 0;
    end
    d.c = 8'd200;                    // now unsatisfiable as state
    unsat = d.randomize(a);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "ok_pair"), 1, "only the named members are solved");
    assert_eq!(u(&sim, "c_kept"), 30, "the unlisted member is untouched");
    assert_eq!(
        u(&sim, "ok_all"),
        1,
        "the whole-object form still solves everything"
    );
    assert_eq!(
        u(&sim, "unsat"),
        0,
        "an unsatisfiable state member fails the call"
    );
}
