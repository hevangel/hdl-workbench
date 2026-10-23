//! §6.20.3 / §23.2.3 — a MODULE `type` parameter used inside a behavioral
//! block of the parameterized module (`CLASS_T obj; obj = new();`).
//!
//! Type-parameter bindings were registered into the global typedef table only
//! transiently during each instance's inlining and RESTORED afterwards (they
//! are per-instance, so one global name cannot hold both `s_def`'s and
//! `s_ovr`'s bindings). The cloned initial/always statements therefore had no
//! binding left at run time: `obj = new()` never resolved the concrete class
//! and every property read came back x — for the declared DEFAULT and the
//! `#(.CLASS_T(..))` override alike.
//!
//! Each instance's bindings (overrides and defaults, chased through typedefs)
//! now ride on the inliner's `RewriteCtx`, and `materialize` substitutes the
//! resolved concrete type into cloned declarations.

use xezim::simulate;

const SRC: &str = r#"
module tb;
  sub                 s_def ();
  sub #(.CLASS_T(other)) s_ovr ();
  int def_id, def_direct, ovr_id, prim_bits;
  initial begin
    #2;
    def_id     = s_def.probe_obj;
    def_direct = s_def.probe_direct;
    ovr_id     = s_ovr.probe_obj;
    prim_bits  = s_ovr.probe_prim_bits;
  end
endmodule
class payload;
  int id = 32'hDEADBEEF;
endclass
class other;
  int id = 32'h5A5A5A5A;
endclass
module sub #(parameter type CLASS_T = payload, parameter type PRIM_T = int);
  int probe_obj, probe_direct, probe_prim_bits;
  initial begin
    CLASS_T obj;
    payload direct;
    PRIM_T  prim;
    obj = new();
    direct = new();
    probe_obj       = obj.id;
    probe_direct    = direct.id;
    probe_prim_bits = $bits(prim);
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn module_type_param_class_handles_construct_per_instance() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "def_id"),
        0xDEAD_BEEF,
        "defaulted CLASS_T constructs the default class"
    );
    assert_eq!(
        u(&sim, "def_direct"),
        0xDEAD_BEEF,
        "direct class handle (control)"
    );
    assert_eq!(
        u(&sim, "ovr_id"),
        0x5A5A_5A5A,
        "overridden CLASS_T constructs the override class"
    );
    assert_eq!(
        u(&sim, "prim_bits"),
        32,
        "a primitive type param still sizes locals"
    );
}
