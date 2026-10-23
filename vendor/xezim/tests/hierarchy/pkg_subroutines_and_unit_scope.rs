//! §26.3 / §22.4 audit fixes, reference-validated.
//!
//! 1. `pkg::f()` must call THAT package's subroutine: package functions and
//!    tasks now register under `pkg::name` alongside the bare hoisted key,
//!    and the runtime prefers the qualified one on a scoped call.
//! 2. §3.12.1: a module-local declaration shadows a same-named $unit
//!    variable — two DISTINCT objects. `$unit::name` reaches the unit copy
//!    from inside the shadowing module, and a $unit subroutine's body keeps
//!    ITS references bound to the unit copy (not the module's shadow).
//!    Known gap (documented): the $unit variable is per-module storage, so a
//!    mutation made inside one module is not visible from another module's
//!    unshadowed reference.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn scoped_package_subroutines_do_not_collide() {
    let src = r#"
package p1;
  function automatic int get(); return 111; endfunction
  localparam int K = 1;
endpackage
package p2;
  function automatic int get(); return 222; endfunction
  localparam int K = 2;
endpackage
module tb;
  int a, b, c, d;
  initial begin
    a = p1::get();
    b = p2::get();
    c = p1::K;
    d = p2::K;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 111, "p1::get must reach p1's function");
    assert_eq!(
        u(&sim, "b"),
        222,
        "p2::get must reach p2's, not the hoisted bare one"
    );
    assert_eq!(u(&sim, "c"), 1);
    assert_eq!(u(&sim, "d"), 2);
}

#[test]
fn unit_scope_shadowing_and_qualified_access() {
    let src = r#"
int ucount = 100;
function automatic int unext();
  ucount = ucount + 1;
  return ucount;
endfunction
module leaf;
  int ucount = 5;          // shadows the $unit variable
  int a, b, c;
  initial begin
    a = ucount;            // 5: local shadows
    b = $unit::ucount;     // 100: qualified reaches the unit copy
    c = unext();           // 101: the $unit function updates the UNIT var
  end
endmodule
module tb;
  leaf u();
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    let g = |n: &str| {
        sim.get_signal(&format!("u.{}", n))
            .unwrap_or_else(|| panic!("missing u.{}", n))
            .to_u64()
            .unwrap()
    };
    assert_eq!(g("a"), 5, "module-local shadows");
    assert_eq!(g("b"), 100, "$unit::name reaches the unit copy");
    assert_eq!(g("c"), 101, "$unit function body stays in $unit scope");
}
