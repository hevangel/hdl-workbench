//! IEEE 1800-2017 §6.18 / §7.2 — a forward-declared class (`typedef class C;`)
//! contributes a HANDLE-sized slot to a struct typedef, just like any other
//! member. Before the fix, the forward-typedef's width-0 sentinel froze into
//! the struct: the class definition never overwrote the placeholder, so
//! `resolve_type_width` returned 0 for the handle member and `$bits` of the
//! struct dropped it entirely.
//!
//! Concretely, with
//!
//!     typedef class C;
//!     typedef struct { C a; int n; } with_handle_t;
//!     class C; int x; endclass
//!
//! `$bits(with_handle_t)` came out equal to `$bits(int)` — the handle
//! contributed nothing — because the forward declaration registered `C -> 0`
//! and nothing later replaced it. The handle is genuinely a 32-bit slot, so
//! the struct must be STRICTLY WIDER than the int alone.
//!
//! Companion reproducer for aionhw/xezim-core#15: this test FAILS on
//! `main` (forward-typedef width 0 propagates) and PASSES with the fix (0-width
//! type references fall back to the default handle width). Validated against a
//! reference simulator, which counts the handle (64-bit there) and therefore
//! also reports the struct as wider than the bare int.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("top.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// A forward-declared class member must contribute storage to a struct
/// typedef, so `$bits(with_handle_t) > $bits(only_int_t)`.
const SRC: &str = r#"
package p_pkg;
  typedef class C;
  typedef struct { C a; int n; } with_handle_t;
  typedef struct { int n; } only_int_t;
  class C;
    int x;
  endclass
endpackage

module top;
  import p_pkg::*;
  int bw, bo, pass;
  with_handle_t s;
  initial begin
    bw = $bits(with_handle_t);
    bo = $bits(only_int_t);
    pass = 0;

    // The class-handle member MUST contribute storage: a struct with the
    // handle must be strictly wider than one holding only the int.
    if (bw > bo) pass = pass + 1;

    // Functional round-trip: both members survive a struct-sized value.
    s.n = 42;
    s.a = new;
    s.a.x = 7;
    if (s.n == 42 && s.a.x == 7) pass = pass + 1;

    $display("BITS_WITH=%0d BITS_ONLY=%0d", bw, bo);
    if (pass == 2) $display("TAG_PASS"); else $display("TAG_FAIL");
  end
endmodule
"#;

#[test]
fn forward_typedef_class_handle_counts_in_struct_bits() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // Both the width contribution and the round-trip must hold.
    assert_eq!(
        get(&sim, "pass"),
        2,
        "forward-typedef class handle dropped from struct width"
    );
    // And the handle member must make the struct strictly wider than the
    // int alone (the bug made them equal).
    assert!(
        get(&sim, "bw") > get(&sim, "bo"),
        "class-handle member contributed no storage: with={} only={}",
        get(&sim, "bw"),
        get(&sim, "bo")
    );
}
