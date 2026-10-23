//! Compiled-method static-local persistence (IEEE 1800-2017 §6.21).
//!
//! A `static` local inside a class method used to be lowered by the
//! method-block compiler as if it were AUTOMATIC: the block seeded the
//! register with a constant, `fold_const_regs` folded every comparison
//! against that seed (`m_inst == null` folded to constant `true`), and
//! the block re-ran the initializer on EVERY call. For
//!
//! ```systemverilog
//! static cache m_inst;
//! if (m_inst == null) m_inst = new();
//! return m_inst;
//! ```
//!
//! every compiled call returned a FRESH object: the singleton invariant
//! was silently lost, and the method got slower by exactly the cost of
//! the construction (an ~10x slowdown for any class holding an unpacked
//! array property — the array merely made `new()` expensive enough to
//! notice).
//!
//! The compiler now records static locals in `CompiledBlock::static_locals`
//! (registering a persistence cell instead of emitting a seed), and the
//! VM seeds/copies the registers through the same class/spec/subroutine-
//! qualified keys the AST path uses, so both paths share one cell.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

/// The singleton shape: many calls must return ONE object, and the
/// initializer must run exactly once — even on the tiered-to-compiled
/// path (the run calls the method far more than the default tier
/// threshold so the compiled block takes over mid-run; the identity
/// must not change at the switchover).
#[test]
fn compiled_static_local_singleton_persists() {
    const SRC: &str = "module tb;
  class cache;
    string m_leaves[8];
    int m_id;
    function new(int id);
      m_id = id;
    endfunction
  endclass
  class holder;
    static cache m_inst;
    static int ctor_calls;
    static function cache get_inst();
      if (m_inst == null) begin
        m_inst = new(ctor_calls);
        ctor_calls++;
      end
      return m_inst;
    endfunction
  endclass
  cache first;
  int mismatches;
  int calls;
  int first_id;
  initial begin
    first = holder::get_inst();
    for (calls = 0; calls < 6000; calls++) begin
      if (holder::get_inst() != first) mismatches++;
    end
    first_id = first.m_id;
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "mismatches"),
        0,
        "every call returns the SAME singleton"
    );
    assert_eq!(
        u(&sim, "first_id"),
        0,
        "the initializer ran once (first construction)"
    );
}

/// A scalar static local must also persist across compiled calls
/// (a call counter is the classic shape).
#[test]
fn compiled_static_local_scalar_counter() {
    const SRC: &str = "module tb;
  class counter;
    static int tick;
    static function int next();
      tick++;
      return tick;
    endfunction
  endclass
  int sum;
  int i;
  initial begin
    for (i = 0; i < 6000; i++) sum += counter::next();
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    // sum of 1..=6000 = 6000*6001/2 = 18_003_000
    assert_eq!(
        u(&sim, "sum"),
        18_003_000,
        "static counter persisted across every call"
    );
}

/// The AST path (compiled methods disabled) must keep the same
/// singleton semantics — the two paths share the persistence cells.
#[test]
fn static_local_singleton_ast_path_agrees() {
    const SRC: &str = "module tb;
  class cache;
    int m_id;
    function new(int id);
      m_id = id;
    endfunction
  endclass
  class holder;
    static cache m_inst;
    static int ctor_calls;
    static function cache get_inst();
      if (m_inst == null) begin
        m_inst = new(ctor_calls);
        ctor_calls++;
      end
      return m_inst;
    endfunction
  endclass
  cache first;
  int mismatches;
  int calls;
  int first_id;
  initial begin
    first = holder::get_inst();
    for (calls = 0; calls < 300; calls++) begin
      if (holder::get_inst() != first) mismatches++;
    end
    first_id = first.m_id;
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "mismatches"), 0, "AST path singleton holds");
    assert_eq!(u(&sim, "first_id"), 0);
}
