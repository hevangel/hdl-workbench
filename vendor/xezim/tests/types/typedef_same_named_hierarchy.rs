// Self-test: IEEE 1800-2017 §3.13/§6.18 — a TYPEDEF (compilation-unit-scope
// data type) may legally share a name with a hierarchy kind (module,
// interface, program, package, UDP). The reference simulator resolves a
// handle declared with that name to the typedef's target, e.g.
// `typedef base shared;` next to `module shared;` gives a real `base`
// instance for `shared h; h = new();`.
//
// This is the same namespace collision as class-vs-module: the single
// name-keyed `definitions` map gives the slot to the hierarchy kind, so the
// typedef would be erased and the handle resolves to garbage (X). The fix
// keeps every compilation-unit-scope typedef in a dedicated registry
// (`typedef_defs`) and re-runs it through `process_typedef` when its name
// slot was taken by a module/interface/program/package/UDP, so `shared h;`
// still resolves to the `base` class at runtime.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(&format!("top.{}", n))
        .or_else(|| sim.get_signal(n))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

// typedef declared BEFORE the module.
const SRC_A: &str = r#"
class base_k;
  int a;
  function new(); a = 3; endfunction
endclass
typedef base_k shared;      // data-namespace typedef
module shared;              // hierarchy kind shares the name
  integer out;
  initial begin
    shared h;
    h = new();
    out = h.a;
  end
endmodule
"#;

// typedef declared AFTER the module.
const SRC_B: &str = r#"
class base_j;
  int a;
  function new(); a = 13; endfunction
endclass
module shared;              // hierarchy kind claims the name first
  integer out;
  initial begin
    shared h;
    h = new();
    out = h.a;
  end
endmodule
typedef base_j shared;      // data-namespace typedef still coexists
"#;

#[test]
fn typedef_survives_same_named_module() {
    let sim = simulate(SRC_A, 1000).expect("simulate failed");
    // If the module had clobbered the typedef, `shared h; h=new()` could not
    // resolve to `base_k`, so `h.a` would read X instead of 3.
    assert_eq!(
        u(&sim, "out"),
        3,
        "typedef before module: handle must resolve via the typedef"
    );
}

#[test]
fn typedef_survives_module_declared_first() {
    let sim = simulate(SRC_B, 1000).expect("simulate failed");
    // Module-first ordering: the typedef must not evict the module from the
    // name-keyed slot (it must still be the top module) AND must remain
    // resolvable as the handle's data type.
    assert_eq!(
        u(&sim, "out"),
        13,
        "module first: typedef must coexist and resolve"
    );
}
