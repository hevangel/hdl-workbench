//! class-perf: TRIVIAL ACCESSOR INLINING (Fast Accessor)
//!
//! Trivial 1-statement getters (`function int get(); return val; endfunction`)
//! and setters (`function void set(int v); val = v; endfunction`) historically
//! incurred more overhead when compiled (~14% slower than AST) because the
//! VM call frame setup (register swap, frame push/pop, queue frames) cost
//! more than the 1-statement body itself.
//!
//! Fast accessors detect pure member load/store bodies at fast-call entry
//! creation and execute them in-place with zero frame or register allocations,
//! both from AST dispatch (`exec_method_call`) and from VM dispatch
//! (`Insn::CallMethod`).
//!
//! Polymorphism is strictly preserved: derived classes that override an accessor
//! dispatch through the derived implementation, not the base accessor.

use xezim::simulate;

fn gate_on() -> bool {
    super::compiled_method_test_env::eager()
}

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

#[test]
fn fast_accessor_getters_and_setters_roundtrip() {
    if !gate_on() {
        return;
    }
    let src = r#"
class Box;
  int val;
  function void set_val(int v);
    val = v;
  endfunction
  function int get_val();
    return val;
  endfunction
endclass

module top;
  int r0, r1, r2;
  initial begin
    Box b = new();
    b.set_val(42);
    r0 = b.get_val();
    b.set_val(100);
    r1 = b.get_val();
    r2 = r0 + r1; // 142
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 42);
    assert_eq!(u(&sim, "r1"), 100);
    assert_eq!(u(&sim, "r2"), 142);
}

#[test]
fn fast_accessor_polymorphism_override_preserved() {
    if !gate_on() {
        return;
    }
    let src = r#"
class Base;
  int x;
  function void set_x(int v);
    x = v;
  endfunction
  function int get_x();
    return x;
  endfunction
endclass

class Derived extends Base;
  // Overridden getter: must NOT use Base's fast accessor!
  function int get_x();
    return x * 10;
  endfunction
endclass

class Driver;
  Base b1;
  Derived d1;
  function new();
    b1 = new();
    d1 = new();
  endfunction
  function int run(int iters);
    int i, sum = 0;
    for (i = 0; i < iters; i = i + 1) begin
      b1.set_x(i);
      d1.set_x(i);
      sum = sum + b1.get_x() + d1.get_x();
    end
    return sum;
  endfunction
endclass

module top;
  int r;
  initial begin
    Driver drv = new();
    r = drv.run(100); // 54450
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "r"), 54450);
}
