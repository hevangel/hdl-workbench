//! class-perf: compiling wait-free TASK bodies and fast compiled→compiled
//! dispatch for free functions (`Insn::CallFreeFunction`).
//!
//! 1. A class task whose body is wait-free (`for (int i = 0; ...)` loop
//!    with a loop-scoped declaration and a non-trivial body, `$display`
//!    tail) is compiled; the display runs through the AST-fallback insn so
//!    output stays byte-identical to the interpreter and the reference
//!    simulator. Tasks containing waits stay interpreted.
//! 2. A compiled method delegating to free functions dispatches through
//!    the direct VM path: string formals, string handling, `bit` results
//!    (param-able return type on a free fn is admissible because the scope
//!    is receiverless and deterministic), default-valued formals (arity
//!    mismatch falls back to the interpreter binding loop), and a nested
//!    free-fn chain f→g must all behave exactly as interpreted.
//!
//! Expectations verified byte-for-byte against reference simulators.

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 200).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Task with loop-scoped `int i`, non-simple body, `$display` tail, plus
/// break/continue in a second task and a wait task that must NOT compile.
#[test]
fn task_for_vardecl_display_and_break() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
class C;
  int hits;
  function bit ok(int k); return (k % 2) == 0; endfunction
  task hot(int n);
    for (int i = 0; i < n; i++) begin
      if (ok(i)) hits++;
    end
    $display("HITS=%0d", hits);
  endtask
  task brk(int n);
    int seen;
    for (int i = 0; i < n; i++) begin
      if (i == 5) continue;
      if (i == 8) break;
      seen += i;
    end
    $display("SEEN=%0d", seen);
  endtask
  task downs(int n);
    int acc;
    for (int i = n; i > 0; i--) acc += i;
    $display("ACC=%0d", acc);
  endtask
  task slow();
    #1 $display("SLOW");
  endtask
endclass
module top;
  C c;
  initial begin
    c = new();
    c.hot(10);
    c.brk(12);
    c.downs(4);
    c.slow();
    $display("DONE");
  end
endmodule
"#;
    let o = out(src);
    assert!(o.contains("HITS=5"), "loop-scoped vardecl task body:\n{o}");
    assert!(
        o.contains("SEEN=23"),
        "break/continue in for-vardecl loop:\n{o}"
    );
    assert!(o.contains("ACC=10"), "count-down for-vardecl loop:\n{o}");
    assert!(
        o.contains("SLOW"),
        "wait task must still run (interpreted):\n{o}"
    );
    assert!(o.contains("DONE"), "tail display after tasks:\n{o}");
}

/// Compiled method → free-function chain through the direct dispatch:
/// string formals, nested chain, default formal arity, `bit` result.
#[test]
fn free_fn_chain_fast_dispatch() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
function automatic int f1(string s, int add);
  return s.len() + add;
endfunction
function automatic int f2(string s);
  if (s == "abc") return f1(s, 100);
  return f1(s, 7) * 2;
endfunction
function automatic bit fstr(string s, int dummy = 42);
  return f2(s) > 10;
endfunction
class C;
  int a, b, c, d;
  task hot(int n);
    for (int i = 0; i < n; i++) begin
      a = f2("abc");
      b = f2("xy");
      c = fstr("abcd");
      d = fstr("ab");   // (2+7)*2 = 18 > 10 -> still 1; exercises defaults
    end
  endtask
endclass
module top;
  C h;
  initial begin
    h = new();
    h.hot(2000);
    $display("A=%0d B=%0d C=%0d D=%0d", h.a, h.b, h.c, h.d);
    if (h.a == 103 && h.b == 18 && h.c == 1 && h.d == 1)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL");
  end
endmodule
"#;
    let o = out(src);
    assert!(
        o.contains("A=103 B=18 C=1 D=1"),
        "free-fn chain results:\n{o}"
    );
    assert!(o.contains("TAG_PASS"), "chain verdict:\n{o}");
}

/// The UVM-shaped regression: `for` loop over `uvm_is_match`-style string
/// predicates driven from a compiled task, with a failing branch built like
/// a report macro (begin-block with a call), ensuring the branch compiles
/// even though it never fires.
#[test]
fn task_loop_string_predicate_macro_shape() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let src = r#"
function automatic bit is_pre(string p);
  return p.substr(0, 2) == "foo";
endfunction
class reporter;
  int fatals;
  function void fatal(string id);
    fatals++;
    $display("FATAL[%s]", id);
  endfunction
endclass
class C;
  reporter rep;
  task run_it(int n);
    for (int i = 0; i < n; i++) begin
      if (!is_pre($sformatf("foo_%0d", i))) begin
        rep.fatal("mismatch");
      end
    end
  endtask
endclass
module top;
  C c;
  initial begin
    c = new();
    c.rep = new();
    c.run_it(500);
    $display("FATALS=%0d", c.rep.fatals);
    if (c.rep.fatals == 0)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL");
  end
endmodule
"#;
    let o = out(src);
    assert!(o.contains("FATALS=0"), "no false fatals:\n{o}");
    assert!(o.contains("TAG_PASS"), "verdict:\n{o}");
}
