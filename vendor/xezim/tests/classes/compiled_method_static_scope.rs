//! class-perf Step 9h: CLASS-SCOPE static calls (`Cls::m(args)` /
//! `alias::m(args)`, and the lax `Cls.m(args)` dot form the parser renders
//! the same way) compile inside class-method bytecode as
//! `Insn::CallStaticScoped`.
//!
//! The receiver Ident is admitted by the compile site when it names an
//! elaborated class, a declaring-chain typedef alias, or a type parameter,
//! and is NOT any variable-like name (local, member, shadow, signal). The
//! RAW name is baked into the insn; the executor re-enters the
//! interpreter's shared `exec_class_scope_static_call`, so specialization
//! derivation, per-spec static seeding, typedef resolution and the `new`
//! fallback stay runtime-owned — byte-identical to the AST path.
//!
//! Companion behavior pinned here:
//! - a compiled STATIC body must not read a class VALUE parameter through
//!   a baked `LoadClassStatic` (that reads the default specialization);
//!   such callees decline and run on the AST, which resolves the param
//!   from the active specialization (the alias-of-`C#(7)` shape: 7/3, not 4/4);
//! - a function whose IMPLICIT return local has class type gets its cell
//!   seeded from the frame, so `$cast(funcname, obj)` inside a compiled
//!   body keeps the handle (the sequencer create_item regression shape).
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

/// Plain class-name receiver, chain typedef alias receiver, and a
/// class-scope call returning a handle whose member is read. All calls sit
/// inside a compiled instance method (`go`), which previously declined on
/// the member-call receiver and now lowers CallStaticScoped. Verified
/// byte-for-byte against reference simulators (a=20 b=10 d=42).
#[test]
fn class_scope_static_calls() {
    if !gate_on() {
        return;
    }
    let src = r#"
module top;
  class Util;
    static int n;
    static function int twice(int v); n = n + 1; return v * 2; endfunction
    static function Util me();
      static Util m;
      if (m == null) m = new();
      return m;
    endfunction
    int id;
    function new(); id = 42; endfunction
  endclass
  class W;
    typedef Util alias_t;
    int r1, r2, r4;
    function void go();
      r1 = Util::twice(10);
      r2 = alias_t::twice(5);
      r4 = Util::me().id;
    endfunction
  endclass
  W w;
  int a, b, d;
  initial begin
    w = new();
    w.go();
    a = w.r1; b = w.r2; d = w.r4;
  end
endmodule
"#;
    let mut sim = simulate(src, 100).expect("simulate");
    sim.run();
    assert_eq!(u(&sim, "a"), 20);
    assert_eq!(u(&sim, "b"), 10);
    assert_eq!(u(&sim, "d"), 42);
}

/// Module- and class-scope typedef aliases to PARAMETERIZED
/// specializations (`typedef C#(7) C7_t;`). The static callee reads a
/// class VALUE parameter: it declines under the gate (its param is
/// specialization-scoped at run time) and the AST resolves it from the
/// active spec. Verified byte-for-byte against reference simulators
/// (a=7 b=3 — a wrong baked default would give 4/4).
#[test]
fn param_spec_alias_receivers() {
    if !gate_on() {
        return;
    }
    let src = r#"
module top;
  class C #(int W = 4);
    static function int get_w(); return W; endfunction
  endclass
  typedef C#(7) C7_t;
  class Outer;
    typedef C#(3) C3_t;
    int r1, r2;
    function void go();
      r1 = C7_t::get_w();
      r2 = C3_t::get_w();
    endfunction
  endclass
  Outer o;
  int a, b;
  initial begin
    o = new();
    o.go();
    a = o.r1; b = o.r2;
  end
endmodule
"#;
    let mut sim = simulate(src, 100).expect("simulate");
    sim.run();
    assert_eq!(u(&sim, "a"), 7);
    assert_eq!(u(&sim, "b"), 3);
}

/// A singleton-get class-scope call chain: every call through the alias
/// must construct exactly one object. Verified byte-for-byte against
/// reference simulators (a=1 b=1).
#[test]
fn alias_singleton_call_chain() {
    if !gate_on() {
        return;
    }
    let src = r#"
module top;
  class C;
    static C me;
    int id;
    static int n;
    function new(); n = n + 1; id = n; endfunction
    static function C get();
      if (me == null) begin
        me = new();
      end
      return me;
    endfunction
  endclass
  class W;
    typedef C alias_t;
    int r1, r2;
    function void go();
        C c;
        c = alias_t::get();
        r1 = c.id;
        c = alias_t::get();
        r2 = c.id;
      endfunction
  endclass
  W w;
  int a, b;
  initial begin
    w = new();
    w.go();
    a = w.r1;
    b = w.r2;
  end
endmodule
"#;
    let mut sim = simulate(src, 100).expect("simulate");
    sim.run();
    assert_eq!(u(&sim, "a"), 1);
    assert_eq!(u(&sim, "b"), 1);
}

/// A class-returning method whose body casts into the implicit return
/// local: `$cast(funcname, factory_result)` inside a COMPILED body keeps
/// the full handle when the function's result has class type (the cell is
/// seeded from the frame). Regression shape of the sequencer zero-time
/// loop: a null create_item wedged every driver.
#[test]
fn class_result_cast_retains_handle() {
    if !gate_on() {
        return;
    }
    let src = r#"
module top;
  class Item;
    int tag;
    function new(); tag = 5; endfunction
  endclass
  class Maker;
    static int built;
    function Item make();
      Item tmp;
      tmp = new();
      built = built + 1;
      void'($cast(make, tmp));
      return make;
    endfunction
  endclass
  Maker mk;
  Item it;
  int a, b;
  initial begin
    mk = new();
    it = mk.make();
    if (it != null) begin
      a = it.tag;
    end else begin
      a = 0;
    end
    b = Maker::built;
  end
endmodule
"#;
    let mut sim = simulate(src, 100).expect("simulate");
    sim.run();
    assert_eq!(u(&sim, "a"), 5, "create-item handle must not be nulled");
    assert_eq!(u(&sim, "b"), 1);
}
