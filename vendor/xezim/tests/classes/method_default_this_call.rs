//! A method formal's DEFAULT argument expression is evaluated at call time in
//! the callee's scope (§13.3.1/§13.5.3), so it may reference `this` or call a
//! sibling method on the receiving instance.
//!
//! Distilled from UVM's parameterized `uvm_event#(T)`:
//!
//! ```systemverilog
//! virtual function void trigger(T data = get_default_data());
//! ```
//!
//! When `trigger()` is called with no argument, the omitted formal is filled
//! by evaluating `get_default_data()` on THAT instance — the same `this` the
//! call is dispatched on. The simulator evaluated the default in the CALLER's
//! scope before the callee's `this`/class context was pushed, so the
//! `this`-dependent method default silently evaluated to the type's zero value
//! (a null handle) instead of the instance's stored default data.
//!
//! `myevent#(T)` mirrors the shape: `trigger(T data = get_default_data())`
//! and `set_default_data`. The test sets a non-trivial default, triggers with
//! no argument, and checks that the trigger recorded exactly the instance's
//! default — proving the default was evaluated against `this`, not zeroed.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

#[test]
fn method_default_evaluates_this_dependent_call() {
    const SRC: &str = "class myevent #(type T = int);
  local T m_default;
  local T m_last;

  virtual function T get_default_data();
    return m_default;
  endfunction

  virtual function void set_default_data(T v);
    m_default = v;
  endfunction

  virtual function void trigger(T data = get_default_data());
    m_last = data;
  endfunction

  virtual function T get_trigger_data();
    return m_last;
  endfunction
endclass

module tb;
  int dflt_ok;
  int trig_ok;
  myevent#(int) ev;
  initial begin
    ev = new();
    // default m_d is 0; trigger() with no arg must capture 0
    ev.trigger();
    if (ev.get_trigger_data() == 0) dflt_ok = 1;
    // now set a default and trigger() with no arg must capture THIS instance's default
    ev.set_default_data(42);
    ev.trigger();
    if (ev.get_trigger_data() == 42) trig_ok = 1;
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "dflt_ok"),
        1,
        "default arg evaluated via get_default_data (zero case)"
    );
    assert_eq!(
        u(&sim, "trig_ok"),
        1,
        "default arg evaluated via get_default_data (set case)"
    );
}
