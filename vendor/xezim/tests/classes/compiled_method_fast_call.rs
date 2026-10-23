//! class-perf P3 — DIRECT VM->VM DISPATCH (`Insn::CallMethod` fast path).
//!
//! When a compiled method calls another compiled method, the simulator can
//! seed the callee's register file from the caller's argument registers
//! instead of materializing sized-hex constant `Expression`s and re-running
//! the interpreter's binding loop. The fast path must be OBSERVABLY
//! identical to the historic route for everything the guards admit:
//!
//! * nested compiled->compiled calls (2nd+ call — the first goes through
//!   the interpreter and builds the plan/block caches);
//! * formal coercion: `byte`/`int` signedness stamps and width resizes the
//!   binding loop would have applied;
//! * class-handle formals and class-handle results (chained dispatch);
//! * string formals (the sized-hex round-trip this path eliminates);
//! * `function void` callees (seeded result cell);
//! * arity mismatch (default formal) — must take the interpreter route and
//!   still bind the default;
//! * recursion through the fast path.
//!
//! Expectations are reference-simulator-validated. Every test runs with
//! `XEZIM_METHOD_TIER=0` (compile on first call) so the fast path engages
//! on the second call of each callee.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

/// The basic nested-call shape: both methods compile; from the second
/// caller invocation on, the callee runs through the direct path.
#[test]
fn nested_call_result_and_arg_passing() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class C;
  function int add3(int a, int b, int c); return a + b + c; endfunction
  function int twice_sum(int a, int b); return add3(a, b, 0) * 2; endfunction
endclass
module tb;
  integer r0, r1, r2;
  initial begin
    C c = new();
    r0 = c.twice_sum(1, 2);       // first call: callee block being built
    r1 = c.twice_sum(10, 20);     // second+: direct VM->VM
    r2 = c.twice_sum(-5, 5);      // signed actuals through the fast path
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 6);
    assert_eq!(u(&sim, "r1"), 60, "direct-path result");
    assert_eq!(u(&sim, "r2"), 0, "signed actuals: (-5+5)*2 == 0");
}

/// The binding loop stamps signedness and resizes widths for integral
/// formals; the fast path must apply the same coercion.
#[test]
fn formal_coercion_signed_and_width() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class C;
  // byte formal: 8-bit signed - a 32-bit -1 must arrive as -1 (8'hff)
  function int get_byte(byte b); return b; endfunction
  // int formal must read as SIGNED even when the caller passes a wide
  // unsigned register value
  function int is_neg(int v); return (v < 0); endfunction
  function int run(byte bb); return get_byte(bb); endfunction
  function int run_neg(int w); return is_neg(w); endfunction
endclass
module tb;
  integer e0, e1, e2, e3;
  initial begin
    C c = new();
    e0 = c.run(-1);              // first: interpreter
    e1 = c.run(-1);              // second+: direct - byte(-1) stays -1
    e2 = c.run_neg(-7);          // first
    e3 = c.run_neg(-7);          // direct: int -7 < 0
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "e0"),
        0xFFFF_FFFF,
        "interpreter route byte coercion"
    );
    assert_eq!(u(&sim, "e1"), 0xFFFF_FFFF, "direct-path byte coercion");
    assert_eq!(u(&sim, "e2"), 1, "interpreter route signed stamp");
    assert_eq!(u(&sim, "e3"), 1, "direct-path signed stamp");
}

/// Class-handle formals and handle results through the direct path:
/// `wrap(h).inner().id()` chains three compiled frames.
#[test]
fn handle_formals_and_chained_handle_results() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class node;
  int id;
  function new(int i); id = i; endfunction
  function int get_id(); return id; endfunction
endclass
class C;
  function node inner(node n); return n; endfunction
  function node wrap(node n); return inner(n); endfunction
  function int id_of(node n); return wrap(n).get_id(); endfunction
endclass
module tb;
  integer r0, r1, r2;
  initial begin
    C c = new();
    node a = new(11);
    node b = new(22);
    r0 = c.id_of(a);
    r1 = c.id_of(b);
    r2 = c.id_of(a);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "r0"),
        11,
        "handle arg + handle result + chained call"
    );
    assert_eq!(u(&sim, "r1"), 22);
    assert_eq!(u(&sim, "r2"), 11);
}

/// String formals — the direct path passes the packed bytes untouched
/// instead of the sized-hex constant round-trip.
#[test]
fn string_formal_through_direct_path() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class C;
  function string dup(string s); return {s, s}; endfunction
  function string twice(string s); return dup(s); endfunction
endclass
module tb;
  integer n0, n1, n2;
  initial begin
    C c = new();
    n0 = c.twice("ab").len();
    n1 = c.twice("ab").len();
    n2 = c.twice("").len();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "n0"), 4, "interpreter route");
    assert_eq!(u(&sim, "n1"), 4, "direct path: \"ab\"->\"abab\"");
    assert_eq!(u(&sim, "n2"), 0, "empty string round-trips");
}

/// `function void` callees run on the direct path too (seeded result cell,
/// member stores still visible).
#[test]
fn void_callee_side_effects_visible() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class C;
  int acc;
  function new(); acc = 0; endfunction
  function void bump(int d); acc = acc + d; endfunction
  function void bump_twice(int d); bump(d); bump(d); endfunction
endclass
module tb;
  integer a0, a1, a2;
  initial begin
    C c = new();
    c.bump_twice(5);  a0 = c.acc;   // first: interpreter
    c.bump_twice(7);  a1 = c.acc;   // direct path
    c.bump_twice(1);  a2 = c.acc;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a0"), 10);
    assert_eq!(u(&sim, "a1"), 24, "void direct-path member stores");
    assert_eq!(u(&sim, "a2"), 26);
}

/// Fewer actuals than formals (default formal) must take the interpreter
/// route and bind the default exactly.
#[test]
fn default_formal_binds_through_interpreter_route() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class C;
  function int f(int a, int b = 40); return a + b; endfunction
  function int g(int a); return f(a); endfunction
  function int h(int a); return f(a, 2); endfunction
endclass
module tb;
  integer r0, r1, r2;
  initial begin
    C c = new();
    r0 = c.g(1);    // arity mismatch inside g: interpreter route + default
    r1 = c.g(2);
    r2 = c.h(3);    // exact arity: direct path after first call
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 41, "default formal bound (interpreter)");
    assert_eq!(u(&sim, "r1"), 42);
    assert_eq!(u(&sim, "r2"), 5, "exact arity after default call");
}

/// Recursion runs through the direct path (each frame gets its own
/// register file, mirroring `try_run_compiled_method`).
#[test]
fn recursion_through_direct_path() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class C;
  function int fib(int n);
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
  endfunction
endclass
module tb;
  integer r0, r1;
  initial begin
    C c = new();
    r0 = c.fib(12);   // first: interpreter + compile; recursion direct after
    r1 = c.fib(15);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 144, "fib(12)");
    assert_eq!(u(&sim, "r1"), 610, "fib(15)");
}
