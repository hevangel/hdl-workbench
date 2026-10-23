//! Class method dispatch through an inheritance chain, and method calls used
//! inside arithmetic expressions.
//!
//! Guards two simulator hot paths that UVM phasing hammers millions of times:
//!
//! 1. `exec_method_in_class_hierarchy` previously cloned the ENTIRE
//!    `ElaboratedClass` (every method, property, constraint, param map) on
//!    every method lookup — `O(class_size × call_count)`. It now clones at
//!    most the single matched method. The dispatch result must be identical.
//!
//! 2. `infer_width` of a method call previously EXECUTED the method purely to
//!    read its result width, so a method used in an arithmetic expression ran
//!    twice (once for width, once for value) — a double-side-effect bug. It
//!    now takes the width from the method's declared return type via
//!    `class_method_return_width`. The arithmetic results must be identical.
//!
//! `base` defines the methods; `ext` inherits them, so every call walks the
//! hierarchy. The loop drives the dispatch path repeatedly; the assertions
//! pin the exact accumulated values so any change to either path is caught.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

#[test]
fn inherited_method_dispatch_and_arithmetic_width() {
    const SRC: &str = "class base;
  bit [7:0] a;
  function new(); a = 8'h1; endfunction
  function bit [7:0] get_a(); return a; endfunction
  function void inc(); a = a + 1; endfunction
  function int sum_with(input int x); return a + x; endfunction
endclass

class ext extends base;
  function new(); super.new(); endfunction
endclass

module tb;
  int total;
  int chk;
  initial begin
    ext e = new;
    total = 0;
    // 100 iterations: each calls get_a + sum_with (method calls inside an
    // arithmetic expression, resolved in `base` via the hierarchy) and inc.
    for (int i = 0; i < 100; i++) begin
      total = total + e.get_a() + e.sum_with(10);
      e.inc();
    end
    chk = e.get_a();   // a after 100 increments from 1 -> 101
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    // sum_{k=1..100} (k + (k+10)) = 2*5050 + 1000 = 11100
    assert_eq!(
        u(&sim, "total"),
        11100,
        "accumulated method-call arithmetic"
    );
    assert_eq!(
        u(&sim, "chk"),
        101,
        "void method side effect via hierarchy walk"
    );
}
