// Self-test: IEEE 1800-2017 §3.12/§3.13 — a CLASS and an INTERFACE may
// legally share a name. A plain (non-`virtual`) block-local declaration
// `shared h;` is a CLASS HANDLE, not a virtual interface, even though the
// name also denotes an interface.
//
// Before the fix, the runtime's virtual-interface detector
// (`is_virtual_iface_type`) classified ANY `TypeReference` whose name was
// also an interface as a virtual interface — it never checked whether the
// name was ALSO a class. With `interface shared;` + `class shared;`, a
// procedural `shared h; h = new(5);` was therefore intercepted as a virtual
// interface BINDING rather than a class construction: the `new(5)` ran as a
// generic call and stored a null handle. Reading `h.x` then faulted with a
// null-object dereference, whereas the same collision with a *module*
// (`class shared` + `module shared`) worked correctly.
//
// The fix makes `is_virtual_iface_type` prefer the CLASS when a plain
// `TypeReference` name is BOTH an interface and a class — only an explicit
// `virtual <iface>` unambiguously denotes a virtual interface.

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
class shared;            // class shares the interface's name
  int x;
  function new(int v); x = v; endfunction
endclass

interface shared;        // same name — legal, coexists with the class
  logic y;
endinterface

module top;
  int out;
  initial begin
    shared h;            // plain (non-virtual) block-local: a CLASS handle
    h = new(5);
    out = h.x;
  end
endmodule
"#;

#[test]
fn class_and_interface_can_share_a_name() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    // If the interface had hijacked the plain `shared h;` as a virtual-
    // interface binding, `h` would be null and `h.x` would read as 0
    // (or fault). The class handle must win, so `out` is the x value 5.
    assert_eq!(
        u(&sim, "out"),
        5,
        "a plain same-named class handle must not be treated as a virtual interface"
    );
}
