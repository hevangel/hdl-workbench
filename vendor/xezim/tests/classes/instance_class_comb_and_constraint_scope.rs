//! Three user-testbench defects, all reference-validated. The common thread:
//! each construct was correct in the TOP module and silently wrong inside an
//! INSTANTIATED one, which is exactly where testbench infrastructure lives.
//!
//! 1. **§8.3 — a class declared in an instantiated module was dropped.**
//!    `inline_module_items` had no ClassDeclaration arm: `T x = new();` at a
//!    declaration initializer produced null, and `randomize()` saw no rand
//!    props or constraints (it "succeeded" doing nothing).
//! 2. **§18.3 — a constraint may read a module-scope state variable.** The
//!    validator rejected it outright ("Undeclared identifier in class
//!    constraint"); after inlining, the solver also read the bare name at top
//!    level (nonexistent → x) instead of the instance's variable.
//! 3. **§9.2.2.2 — always_comb sensitivity through a called function.** The
//!    read collector had no `Return` arm, so `return p ^ dep;` hid `dep`;
//!    an instance's function body additionally collected its reads unscoped,
//!    so they resolved to no signal and dropped out of the dep graph.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Class inside an instantiated module: decl-init `new()`, and a constraint
/// that reads the enclosing module's variable (held constant during solving).
#[test]
fn class_in_instance_constraint_reads_module_var() {
    let src = r#"
module eng;
  bit mode;
  class C;
    rand bit [7:0] d;
    constraint c {
      if (mode == 1'b1) { d inside {[8'h00 : 8'h7F]}; }
      else              { d inside {[8'h80 : 8'hFF]}; }
    }
  endclass
  int nn, ok1, lo, ok2, hi;
  initial begin
    C c1 = new();
    nn = (c1 == null);
    mode = 1; ok1 = 1; lo = 1;
    repeat (20) begin
      if (!c1.randomize()) ok1 = 0;
      if (c1.d > 8'h7F) lo = 0;
    end
    mode = 0; ok2 = 1; hi = 1;
    repeat (20) begin
      if (!c1.randomize()) ok2 = 0;
      if (c1.d < 8'h80) hi = 0;
    end
  end
endmodule
module tb;
  eng u_e();
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    let g = |n: &str| {
        sim.get_signal(&format!("u_e.{}", n))
            .unwrap_or_else(|| panic!("missing u_e.{}", n))
            .to_u64()
            .unwrap()
    };
    assert_eq!(g("nn"), 0, "decl-init new() must construct (was null)");
    assert_eq!(g("ok1"), 1, "randomize must succeed in mode 1");
    assert_eq!(g("lo"), 1, "mode 1 keeps d <= 0x7F");
    assert_eq!(g("ok2"), 1, "randomize must succeed in mode 0");
    assert_eq!(g("hi"), 1, "mode 0 keeps d >= 0x80");
}

/// §6.18 / §8.25: a scoped parameterized class handle declaration
/// (`pk::W #(.DW(64)) obj;`) parsed as an expression and died at `obj`;
/// a BLOCK-LOCAL typedef of a parameterized class was "undeclared" when used
/// as a static-call scope.
#[test]
fn scoped_class_handle_decl_and_block_typedef() {
    let src = r#"
package pk;
  class W #(parameter int DW = 32);
    bit [DW-1:0] val;
    function new(bit [DW-1:0] v = '0); val = v; endfunction
  endclass
endpackage
module tb;
  class K #(parameter int N = 1);
    static function int get(); return N; endfunction
  endclass
  int b1, v1, b2, g1;
  initial begin
    pk::W #(.DW(64)) obj;
    pk::W deflt;
    typedef K #(.N(7)) k7_t;
    obj = new(64'hDEAD);
    deflt = new(32'hBEEF);
    b1 = $bits(obj.val);
    v1 = obj.val[15:0];
    b2 = $bits(deflt.val);
    g1 = k7_t::get();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "b1"), 64, "parameter override through scoped decl");
    assert_eq!(u(&sim, "v1"), 0xDEAD);
    assert_eq!(u(&sim, "b2"), 32, "default parameters");
    assert_eq!(u(&sim, "g1"), 7, "block-local typedef as static-call scope");
}

/// §9.2.2.2: the block must re-fire when a variable read only inside a called
/// function changes — including a function belonging to an instance.
#[test]
fn always_comb_sensitive_to_function_body_reads() {
    let src = r#"
module leaf(input logic a, output logic o);
  logic dep;
  function automatic logic f(input logic p);
    return p ^ dep;
  endfunction
  always_comb o = f(a);
endmodule
module tb;
  logic a, dep2, out2;
  logic o;
  leaf dut(.a(a), .o(o));
  function automatic logic g(input logic p);
    return p ^ dep2;
  endfunction
  always_comb out2 = g(a);
  int o1, o2, o3, f1, f2;
  initial begin
    a = 0; dep2 = 0; dut.dep = 0;
    #1 o1 = out2;          // 0
    a = 1;
    #1 o2 = out2;          // 1
    dep2 = 1;              // only the function-internal dep changes
    #1 o3 = out2;          // 0
    f1 = o;                // 1 (a=1, dep=0)
    dut.dep = 1;           // instance function's hidden dep
    #1 f2 = o;             // 0
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "o1"), 0);
    assert_eq!(u(&sim, "o2"), 1);
    assert_eq!(u(&sim, "o3"), 0, "dep read only in the function body");
    assert_eq!(u(&sim, "f1"), 1);
    assert_eq!(u(&sim, "f2"), 0, "instance function's hidden dep");
}
