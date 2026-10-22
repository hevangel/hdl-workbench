//! IEEE 1800-2017 §8.7 / §8.12: a class property may have an inline
//! initializer (`T x = new(...);`). SystemVerilog applies member initializers
//! prior to the constructor body. If the constructor does NOT explicitly
//! reassign the property, the inline initializer's constructed object must
//! survive — the property must NOT be null.
//!
//! Previously xezim's object-construction property-init loop skipped any
//! initializer that "contains a call" (including `= new(...)`), on the
//! assumption that such initializers were "the constructor's job." For a
//! property the constructor never touches (e.g. UVM's
//! `my_catcher catcher = new(14);` in a uvm_component whose `new()` never
//! assigns `catcher`), the property stayed at its elaborate-time value (a
//! null handle). This surfaced as UVM "Null callback object cannot be
//! registered" (CBUNREG) in tests that register an inline-constructed
//! callback, and as silent null-handle stalls in phasing/reg-model tests.
//!
//! The fix mirrors the statement-assignment path (`lvalue = new(args)`
//! -> instantiate_class_with_type_args) for the property-init path.
//!
//! Verified byte-for-byte against reference simulators.

use xezim::simulate;

const SRC: &str = r#"
class base;
  int x;
  function new(); x = 5; endfunction
endclass

class simple extends base;
  int v;
  function new(); super.new(); v = 7; endfunction
endclass

class catcher extends base;
  int expected;
  function new(int c); super.new(); expected = c; endfunction
endclass

class holder;
  simple  s1 = new();     // default ctor, parens
  catcher c1 = new(14);   // ctor with arg
  function new(); endfunction
endclass

module top;
  holder h;
  int sv, cv;
  initial begin
    h = new();
    sv = (h.s1 == null) ? -1 : h.s1.v;
    cv = (h.c1 == null) ? -1 : h.c1.expected;
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> i64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n)) as i64
}

#[test]
fn property_new_initializer_constructs() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "sv"),
        7,
        "simple s1 = new() must construct (not null)"
    );
    assert_eq!(
        u(&sim, "cv"),
        14,
        "catcher c1 = new(14) must construct with arg"
    );
}
