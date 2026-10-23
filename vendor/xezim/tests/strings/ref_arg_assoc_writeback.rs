//! `ref` associative-array argument writeback (§13.5 argument-passing semantics).
//!
//! IEEE 1800-2023 §13.5: a `ref` argument aliases the caller's actual — writes
//! performed inside the callee are visible to the caller after return. This
//! worked for scalars, dynamic arrays, and queues, but for **associative
//! arrays** the binding between the formal and the caller's storage was
//! inconsistent and writeback was silently lost in common shapes.
//!
//! The canonical victim was UVM's phase-DAG expansion. `uvm_phase` exposes:
//!
//! ```systemverilog
//!   function void get_successors(ref edges_t successors); // AA by handle
//!     foreach (m_successors[p]) successors[p] = 1;
//!   endfunction
//!   // caller (uvm_phase_hopper::finish_phase):
//!   uvm_phase::edges_t edges;
//!   phase.get_successors(edges);      // ref fill
//!   if (edges.size() != 0) …          // always 0 → no successor NODE scheduled
//! ```
//!
//! `edges` stayed empty, so `build`/`connect`/`run` NODE phases were never
//! scheduled and every data test stalled at time 0 with no phase traversal.
//!
//! Empirically the failure depends on how the formal's name relates to the
//! caller's actual *and* whether the routine is a free function or a method:
//!
//! | routine | formal vs actual name | result |
//! |---------|-----------------------|--------|
//! | method  | different             | **FAIL** (the UVM case) |
//! | method  | same                  | ok |
//! | free fn | same                  | **FAIL** |
//! | free fn | different             | ok |
//!
//! i.e. exactly one direction of each pair loses the writeback. All four
//! cases pass on reference simulators. These tests pin all of them.

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn assert_pass(sim: &xezim::compiler::Simulator, tag: &str) {
    let msgs = messages(sim);
    let pass = msgs.iter().any(|m| m.contains(&format!("{tag}_PASS")));
    let fail = msgs.iter().find(|m| m.contains(&format!("{tag}_FAIL")));
    assert!(
        pass,
        "expected {tag}_PASS in output\nfail line: {fail:?}\nfull output: {msgs:?}"
    );
}

/// The UVM reproducer: a class *method* with a `ref int` AA formal whose name
/// differs from the caller's actual. Writes inside the method must reach the
/// caller. This is the `phase.get_successors(edges)` shape exactly.
const METHOD_DIFF_NAMES: &str = r#"
class Ph;
  function void get(ref int outp[int]);
    outp[1] = 1;
    outp[2] = 1;
  endfunction
endclass
module top;
  initial begin
    Ph h = new;
    int edges[int];
    h.get(edges);
    if (edges.num() == 2)
      $display("METH_PASS num=%0d", edges.num());
    else
      $display("METH_FAIL num=%0d", edges.num());
  end
endmodule
"#;

#[test]
fn method_ref_aa_different_formal_and_actual_name() {
    let sim = simulate(METHOD_DIFF_NAMES, 100).expect("simulate failed");
    assert_pass(&sim, "METH");
}

/// The `m_successors` shape precisely: an AA keyed by class handle, filled by
/// a method through a `ref` formal whose name differs from the caller's.
const HANDLE_KEYED_METHOD: &str = r#"
class Node;
  function new(string n); endfunction
endclass
class Ph;
  function void get(ref int outp[Node], input Node a);
    outp[a] = 1;
  endfunction
endclass
module top;
  initial begin
    Ph h = new;
    Node n = new("A");
    int edges[Node];
    h.get(edges, n);
    if (edges.num() == 1)
      $display("HNDL_PASS num=%0d", edges.num());
    else
      $display("HNDL_FAIL num=%0d", edges.num());
  end
endmodule
"#;

#[test]
fn method_ref_handle_indexed_aa_different_names() {
    let sim = simulate(HANDLE_KEYED_METHOD, 100).expect("simulate failed");
    assert_pass(&sim, "HNDL");
}

/// The free-function flip side: when the caller's actual has the *same* name
/// as the `ref` formal, the free-function path lost writeback.
const FREE_FN_SAME_NAME: &str = r#"
module top;
  function automatic void fill(ref int aa[int]);
    aa[1] = 1;
    aa[2] = 1;
  endfunction
  initial begin
    int aa[int];
    fill(aa);
    if (aa.num() == 2)
      $display("FREE_PASS num=%0d", aa.num());
    else
      $display("FREE_FAIL num=%0d", aa.num());
  end
endmodule
"#;

#[test]
fn free_function_ref_aa_same_formal_and_actual_name() {
    let sim = simulate(FREE_FN_SAME_NAME, 100).expect("simulate failed");
    assert_pass(&sim, "FREE");
}

/// Regression guard: scalar, dynamic-array, and queue `ref` writeback were
/// never affected. This ensures a future fix covers the AA path without
/// regressing the already-working collection kinds.
const NON_AA_REFS: &str = r#"
module top;
  function automatic void f_scalar(ref int s);  s = 99;            endfunction
  function automatic void f_dyn  (ref int d[]); d = new[2]; d[1]=5; endfunction
  function automatic void f_queue(ref int q[$]); q.push_back(7);    endfunction
  initial begin
    int s = 0; int d[]; int q[$];
    f_scalar(s);
    f_dyn(d);
    f_queue(q);
    if (s == 99 && d.size()==2 && d[1]==5 && q.size()==1 && q[0]==7)
      $display("OTHER_PASS s=%0d d.sz=%0d q.sz=%0d", s, d.size(), q.size());
    else
      $display("OTHER_FAIL s=%0d d.sz=%0d q.sz=%0d", s, d.size(), q.size());
  end
endmodule
"#;

#[test]
fn ref_scalar_dynarr_queue_writeback_unaffected() {
    let sim = simulate(NON_AA_REFS, 100).expect("simulate failed");
    assert_pass(&sim, "OTHER");
}
