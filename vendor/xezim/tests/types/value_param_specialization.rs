//! Value parameters of a specialized class, referenced bare inside a STATIC
//! method (no instance constructed). The exact shape of UVM's factory
//! `uvm_object_registry#(T,"base_class")::get_type_name()` → returns the
//! string param `Tname`.
//!
//! Before the fix, a bare reference to a value parameter inside a static
//! method fell through to the class's *declared default* (`<unknown>` / 0)
//! instead of the specialization's actual argument, because no instance is
//! constructed for a static call and the binding only existed in the
//! `#(...)` argument list. `resolve_value_param_from_spec` extracts the
//! argument from the active specialization's signature text and evaluates it
//! (string literals, numeric literals, and bare name references).
//!
//! Verified against reference simulators for the direct-specialization form.
//! (The typedef'd-specialization instance path — `typedef C#(args) Alias;
//! Alias::get()` — is a separate, deeper gap: nested parametric typedef
//! resolution.)

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn assert_contains(sim: &xezim::compiler::Simulator, needle: &str) {
    let msgs = messages(sim);
    assert!(
        msgs.iter().any(|m| m.contains(needle)),
        "expected {:?} in output\nfull output: {msgs:?}",
        needle
    );
}

const STR_PARAM_SPEC: &str = include_str!("../lrm_9_value_param/str_param_spec.sv");

#[test]
fn value_param_from_specialization_static() {
    let sim = simulate(STR_PARAM_SPEC, 100).expect("simulate failed");
    assert_contains(&sim, "STR_PASS 'base_class'");
    assert_contains(&sim, "INT_PASS 7");
    assert_contains(&sim, "DEF_PASS '<unknown>'");
}
