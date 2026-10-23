//! class-perf Step 5 — CLASS-HANDLE RETURNS from class-function methods,
//! plus the bare-member resolution the compiled path needs to lower them.
//!
//! The compiled method path (XEZIM_COMPILE_METHODS=1) must preserve exactly
//! the interpreter's semantics:
//!
//! * a heap handle returned from a getter round-trips UNTOUCHED — no resize,
//!   no signedness stamp (the interpreter never touches a non-`plainly_integral`
//!   Typeref return; a handle is a heap index, not a number);
//! * a chained call `a.get().get()` dispatches on the returned handle;
//! * a null member returns as null (`== null`), not as 0-bit garbage;
//! * a BARE member reference inside a method (`return m_parent;`) resolves to
//!   `this.<member>` — and to the method's DECLARING class's copy when a
//!   derived class shadows the name (§8.10);
//! * members the heap cannot faithfully round-trip (unpacked arrays) keep the
//!   AST semantics — element-wise comparisons stay element-wise.
//!
//! All expectations are reference-simulator-validated (the sources follow the
//! shapes of method_vm9 and the shadowed_property_storage /
//! array_equality_class regressions). Run with the gate OFF and with
//! `XEZIM_COMPILE_METHODS=1` — both must produce identical results.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

#[test]
fn getter_handle_return_roundtrip_and_chaining() {
    let src = r#"
class node;
  int id;
  node parent;
  node child;
  function new(int i); id = i; parent = null; child = null; endfunction
  function node get_parent(); return parent; endfunction
  function node get_child(); return child; endfunction
  function int get_id(); return id; endfunction
endclass
module tb;
  int p, c, ia, ib, ic, ok;
  initial begin
    node a = new(10);
    node b = new(20);
    node n = new(30);
    node got;
    a.child = b;
    b.parent = a;
    // The returned handle must dispatch: b's child chain works.
    got = a.get_child();
    p = got.get_parent().get_id();
    ia = a.get_id();
    ib = got.get_id();
    ic = n.get_id();
    // null member round-trips as null.
    if (n.get_parent() == null) ok = 1; else ok = 0;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "p"),
        10,
        "chained get_child().get_parent() dispatch"
    );
    assert_eq!(u(&sim, "ia"), 10);
    assert_eq!(u(&sim, "ib"), 20, "the returned handle is b, unchanged");
    assert_eq!(u(&sim, "ic"), 30);
    assert_eq!(u(&sim, "ok"), 1, "null member returns as null");
}

/// §8.10 — the compiled bare-member fast path must respect SHADOWED storage:
/// a base method's bare read hits the base's copy, not the leaf's bare key.
#[test]
fn bare_member_read_respects_shadowed_copies() {
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
endclass
module tb;
  int e1, e2, e3, e4;
  initial begin
    D d = new();
    d.setb(55);
    if (d.getb() != 55) e1 = 1; else e1 = 0;
    if (d.getd() != 200) e2 = 1; else e2 = 0;
    d.s = 77;
    if (d.getb() != 55) e3 = 1; else e3 = 0;
    if (d.getd() != 77) e4 = 1; else e4 = 0;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "e1"),
        0,
        "base method reads ITS copy (bare key vs shadow key)"
    );
    assert_eq!(u(&sim, "e2"), 0);
    assert_eq!(
        u(&sim, "e3"),
        0,
        "external leaf write must not leak into the base copy"
    );
    assert_eq!(
        u(&sim, "e4"),
        0,
        "external write must land on the leaf copy"
    );
}

/// Array members must keep AST semantics even when the enclosing method has
/// otherwise-lowerable parts (the compiled path refuses them whole).
#[test]
fn array_member_stays_elementwise() {
    let src = r#"
class C;
  int sa[3];
  function int first(); return sa[0]; endfunction
endclass
module tb;
  initial begin
    C c = new();
    c.sa[0] = 11; c.sa[1] = 22; c.sa[2] = 33;
    if (c.first() != 11) $error("array elem read broken");
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    let errs = sim
        .output
        .iter()
        .filter(|o| o.message.contains("array elem read broken"))
        .count();
    assert_eq!(errs, 0, "array member read through a method diverged");
}
