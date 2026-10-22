//! IEEE 1800-2017 §6.20.2 — `localparam` inside a class body is a local
//! constant, NOT an overridable parameter. A class whose ONLY parameter-like
//! declarations are body `localparam`s is NOT parameterized (has no `#()`
//! header clause), and its static-call initializers (e.g. UVM's
//! `static bit m_reg = SomeClass::register(...)`) MUST run at startup.
//!
//! Previously `class_is_parameterized` also checked `param_defaults`, which
//! contains body `localparam` entries — so it wrongly classified such a class
//! as parameterized, causing its static-call initializers to be SKIPPED
//! (deferred to the per-specialization path that never fires for a
//! non-parameterized class). This broke UVM's `\`uvm_register_cb` for
//! `uvm_report_catcher` (a virtual class with `localparam int DO_NOT_CATCH`),
//! firing CBUNREG warnings in 60+ tests.
//!
//! Verified byte-for-byte against reference simulators.

use xezim::simulate;

const SRC: &str = r#"
class tracker;
  static int count = 0;
  static function bit registered();
    count = count + 1;
    return 1;
  endfunction
endclass

// This class has body localparams but NO header parameters — it is NOT
// parameterized. Its static-call initializer must run at startup.
class has_localparam;
  localparam int DO_NOT_CATCH = 1;
  localparam int DO_NOT_MODIFY = 2;
  static bit m_registered = tracker::registered();
endclass

module top;
  int reg_val;
  int param_val;
  initial begin
    reg_val   = has_localparam::m_registered;
    param_val = has_localparam::DO_NOT_CATCH;
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

#[test]
fn localparam_class_static_init_runs() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // The static-call initializer must have run: count went to 1.
    assert_eq!(
        u(&sim, "reg_val"),
        1,
        "static-call initializer must run for a class with only body localparams"
    );
    // The localparam is accessible and has its value.
    assert_eq!(
        u(&sim, "param_val"),
        1,
        "body localparam must be accessible and correctly valued"
    );
}
