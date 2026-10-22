//! §18.5 — a derived class declared in a MODULE body whose constraint refers
//! to an INHERITED property. Reference-validated.
//!
//! Constraint validation resolves `extends` through the top-level definition
//! map, but a class declared in a module body is a `ModuleItem`, not a
//! top-level definition — so the base was never found, its properties never
//! joined the allowed-identifier set, and the constraint was rejected with
//! "Undeclared identifier", aborting the whole simulation before any output.
//!
//! The identical code at `$unit` or package scope was accepted, and a derived
//! constraint touching only its OWN properties was accepted too, which made
//! this look like a constraint bug rather than a scoping one.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A derived constraint referencing an inherited rand property must elaborate
/// and must actually constrain the solution.
#[test]
fn derived_constraint_uses_an_inherited_property() {
    let src = r#"
module tb;
  class B;
    rand bit [7:0] a;
    constraint c_a { a inside {[10:20]}; }
  endclass
  class D extends B;
    rand bit [7:0] c;
    constraint c_c { c > a; c < 200; }
  endclass
  int ok;
  initial begin
    D d;
    ok = 1;
    d = new();
    for (int i = 0; i < 20; i++) begin
      if (!d.randomize()) ok = 0;
      if (!(d.a >= 10 && d.a <= 20)) ok = 0;
      if (!(d.c > d.a && d.c < 200)) ok = 0;
    end
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "ok"),
        1,
        "the inherited property is in scope and constrains"
    );
}

/// Overriding an inherited constraint by name, and referring to an inherited
/// NON-rand property, both from a module-scope class.
#[test]
fn derived_constraint_override_and_non_rand_reference() {
    let src = r#"
module tb;
  class B;
    rand bit [7:0] a;
    bit [7:0] lim;
    constraint c_a { a inside {[10:20]}; }
  endclass
  class D extends B;
    constraint c_a { a inside {[30:40]}; }   // override by name
  endclass
  class E extends B;
    constraint c_e { a < lim; }              // inherited NON-rand property
  endclass
  int ok_d, ok_e;
  initial begin
    D d; E e;
    ok_d = 1; ok_e = 1;
    d = new();
    for (int i = 0; i < 20; i++) begin
      if (!d.randomize()) ok_d = 0;
      if (!(d.a >= 30 && d.a <= 40)) ok_d = 0;
    end
    e = new();
    e.lim = 8'd15;
    for (int i = 0; i < 20; i++) begin
      if (!e.randomize()) ok_e = 0;
      if (!(e.a < 15)) ok_e = 0;
    end
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "ok_d"),
        1,
        "an overriding constraint replaces the base's"
    );
    assert_eq!(
        u(&sim, "ok_e"),
        1,
        "an inherited non-rand property is in scope"
    );
}
