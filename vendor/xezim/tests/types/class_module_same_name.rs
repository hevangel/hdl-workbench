// Self-test: IEEE 1800-2017 §3.13 — a CLASS and a MODULE may legally share a
// name; they live in different namespaces (hierarchy vs data type). The
// reference simulator accepts `class test;` alongside `module test;` (no
// warning, both resolve).
//
// Before the fix, xezim kept all top-level declarations (modules, interfaces,
// programs, packages, classes) in ONE name-keyed map, and a module with the
// same name as a class OVERWROTE the class entry. The class then vanished
// from `elab.classes`, so a class-typed handle constructed through `new()`
// resolved against the module's (empty) definition and read as garbage / 0.
// This is the shape UVM tests hit when the testbench top module and the UVM
// test class (e.g. `test`) coincidentally share a name.
//
// The fix keeps classes in a dedicated registry (`class_defs`) so a module
// collision cannot evict them; the class is re-registered into `elab.classes`
// under its real name regardless of declaration order.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(&format!("top.{}", n))
        .or_else(|| sim.get_signal(n))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

const SRC: &str = r#"
class test;              // class shares the module's name
  int x;
  function new(int v); x = v; endfunction
  function int get(); return x; endfunction
endclass

module test;             // same name — legal, coexists with the class
  test t;
  int out;
  initial begin
    t = new(42);
    out = t.get();
  end
endmodule
"#;

#[test]
fn class_and_module_can_share_a_name() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    // If the module had clobbered the class, `t.get()` would not see the
    // constructor's value (constructed through the module's empty body) and
    // `out` would be 0 instead of 42.
    assert_eq!(
        u(&sim, "out"),
        42,
        "class-typed handle must not be clobbered by a same-named module"
    );
}
