//! §25.8/§25.9 — virtual interfaces used inside subroutines and class
//! methods. Reference-validated.
//!
//! Everything except `vif.<scalar>` was dead in a subroutine body: nested
//! unpacked members, queue operations, interface subroutine calls. The vif
//! aliasing was per-shape and one level deep — `resolve_hier_name` rewrote a
//! flat dotted Ident, but a subroutine body parses the same source as
//! `MemberAccess`/`Index`/`Call` chains, which no arm matched — so the access
//! fell through to a phantom signal keyed by the literal source text: writes
//! vanished and reads returned 0, silently. This is the standard UVM
//! driver/monitor idiom.
//!
//! There is now ONE rebase: any chain rooted at a bound vif (frame formal,
//! plain vif variable, or class property of the current `this`) is rewritten
//! to the bound instance before evaluation. A bare `v` stays untouched — that
//! is a handle operation (`v2 = v;`, `v == null`).
//!
//! Two sibling defects fixed with it: an interface subroutine or collection
//! builtin called through a plain dotted `MemberAccess` callee (`bi.dbl(4)`,
//! `bi.q.push_back(7)` inside a task, vif or not) had no dispatch arm — the
//! flat-Ident spelling the module-scope rewrite produces worked, the parsed
//! chain did not; and a `virtual` formal of a free FUNCTION was never aliased
//! (only tasks and class methods bound theirs).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

const IFACE: &str = r#"
interface bus_if;
  logic [7:0] data;
  typedef struct { logic [7:0] a; logic [7:0] b; } us_t;
  us_t us;
  int q[$];
  function automatic int dbl(input int x); return x * 2; endfunction
endinterface
"#;

/// A plain interface (no vif at all): queue ops and subroutine calls from
/// inside a task, in the chain spelling the parser produces there.
#[test]
fn interface_collections_and_subroutines_from_a_task() {
    let src = format!(
        "{IFACE}
module tb;
  bus_if bi();
  int t_qsz, t_q0, t_dbl, m_qsz;
  task automatic t;
    bi.q.push_back(7);
    t_qsz = bi.q.size();
    t_q0  = bi.q[0];
    t_dbl = bi.dbl(5);
  endtask
  initial begin
    bi.q.push_back(4);
    m_qsz = bi.q.size();
    t();
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "m_qsz"), 1, "module scope still works");
    assert_eq!(u(&sim, "t_qsz"), 2, "push_back inside the task lands");
    assert_eq!(u(&sim, "t_q0"), 4, "and the element reads back");
    assert_eq!(
        u(&sim, "t_dbl"),
        10,
        "an interface function called from a task"
    );
}

/// The vif itself, held three ways: a module variable used in a task, a task
/// formal, and a free-function formal.
#[test]
fn vif_members_through_tasks_and_function_formals() {
    let src = format!(
        "{IFACE}
module tb;
  bus_if bi();
  virtual bus_if v;
  int a_usa, a_qsz, a_dbl, b_usa, c_fn;
  task automatic use_module_vif;
    v.us.a = 8'h13;
    v.q.push_back(1);
    a_usa = v.us.a;
    a_qsz = v.q.size();
    a_dbl = v.dbl(4);
  endtask
  task automatic use_formal(virtual bus_if w);
    w.us.a = 8'h23;
    b_usa = w.us.a;
  endtask
  function automatic int read_fn(virtual bus_if w);
    return w.us.a + 8'h02;
  endfunction
  initial begin
    v = bi;
    use_module_vif();
    use_formal(v);
    c_fn = read_fn(v);
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "a_usa"),
        0x13,
        "nested unpacked member through a module vif"
    );
    assert_eq!(
        u(&sim, "a_qsz"),
        1,
        "queue op through a module vif in a task"
    );
    assert_eq!(
        u(&sim, "a_dbl"),
        8,
        "interface function through a module vif"
    );
    assert_eq!(u(&sim, "b_usa"), 0x23, "a task's vif formal");
    assert_eq!(
        u(&sim, "c_fn"),
        0x25,
        "a FREE FUNCTION's vif formal is aliased too"
    );
}

/// A vif class property used inside methods — and two objects on two
/// interfaces must not cross.
#[test]
fn vif_class_property_in_methods() {
    let src = format!(
        "{IFACE}
class Agent;
  virtual bus_if vif;
  int got_usa, got_qsz, got_q0;
  function new(virtual bus_if v); vif = v; endfunction
  function void drive(input logic [7:0] x);
    vif.us.a = x;
    vif.q.push_back(x + 1);
  endfunction
  function void observe();
    got_usa = vif.us.a;
    got_qsz = vif.q.size();
    got_q0  = vif.q[0];
  endfunction
endclass
module tb;
  bus_if b0(), b1();
  Agent a0, a1;
  int r0_usa, r0_q0, r1_usa, r1_q0, direct0;
  initial begin
    a0 = new(b0); a1 = new(b1);
    a0.drive(8'h40);
    a1.drive(8'h50);
    a0.observe(); a1.observe();
    r0_usa = a0.got_usa; r0_q0 = a0.got_q0;
    r1_usa = a1.got_usa; r1_q0 = a1.got_q0;
    direct0 = b0.us.a;
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0_usa"), 0x40, "agent 0 reads its own interface");
    assert_eq!(u(&sim, "r0_q0"), 0x41);
    assert_eq!(u(&sim, "r1_usa"), 0x50, "agent 1 does not see agent 0's");
    assert_eq!(u(&sim, "r1_q0"), 0x51);
    assert_eq!(
        u(&sim, "direct0"),
        0x40,
        "the write landed on the real instance"
    );
}
