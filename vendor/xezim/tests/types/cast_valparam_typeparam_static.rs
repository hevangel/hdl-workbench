//! IEEE 1800-2020 §8.25 — `$cast` across **value-parameter** specializations
//! via a type-parameter-typed destination. Inside a parameterized-class
//! static method, `$cast(me, obj)` where `me` is declared `T me;` (T a type
//! parameter bound to a value-parameterized class like `base#(1)`) must
//! reject an object of a *different* value specialization (`base#(2)`).
//!
//! Without the fix, `class_of_var("me")` returned None (T resolves to
//! `base#(1)` which is not in `module.classes`), so `cast_type_ok` fell
//! through to the permissive path and every cross-spec cast succeeded.
//!
//! Verified byte-for-byte against reference simulators.

use xezim::simulate;

const SRC: &str = r#"
module top;
  class base #(int N = 0);
    int id;
    function new(int i); id = i; endfunction
  endclass

  class chk #(type T = base);
    // Mirrors uvm_typed_callbacks#(T)::m_am_i_a
    static function bit am_i_a(base obj);
      T me;
      if (obj == null) return 1;
      return $cast(me, obj);
    endfunction
  endclass

  int same1, cross1, same2, cross2;
  initial begin
    base#(1) b1;
    base#(2) b2;
    b1 = new(11);
    b2 = new(22);
    same1  = chk#(base#(1))::am_i_a(b1);  // expect 1
    cross1 = chk#(base#(1))::am_i_a(b2);  // expect 0
    same2  = chk#(base#(2))::am_i_a(b2);  // expect 1
    cross2 = chk#(base#(2))::am_i_a(b1);  // expect 0
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
fn cross_spec_cast_fails() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "same1"),
        1,
        "same-spec cast (base#1 on base#1) must succeed"
    );
    assert_eq!(
        u(&sim, "cross1"),
        0,
        "cross-spec cast (base#1 on base#2) must fail"
    );
    assert_eq!(
        u(&sim, "same2"),
        1,
        "same-spec cast (base#2 on base#2) must succeed"
    );
    assert_eq!(
        u(&sim, "cross2"),
        0,
        "cross-spec cast (base#2 on base#1) must fail"
    );
}
