//! §9.6.1 — `wait fork` blocks on the calling process's IMMEDIATE children
//! only. Reference-validated.
//!
//! The implementation waited on the transitive descendant closure, and the
//! wake condition recomputed that closure against the live tree — doubly
//! wrong, because a completed child REPARENTS its grandchildren to the waiting
//! parent, so the wait chased the tree as it grew. A `join_none` grandchild
//! extended the wait past every child's completion; a persistent monitor
//! spawned by a child (the standard UVM driver shape) hung it forever. The
//! asymmetry is the LRM's own: `disable fork` kills all descendants, `wait
//! fork` waits on immediate children.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A grandchild outliving its parent must not extend the wait.
#[test]
fn wait_fork_returns_when_children_finish() {
    let src = r#"
module tb;
  int t_after, t_deep;
  task automatic spawn_deep;
    fork begin #6; t_deep = $time; end join_none
  endtask
  task automatic t;
    fork begin #1 spawn_deep(); end join_none
    wait fork;
    t_after = $time;
  endtask
  initial begin
    t();
    #20;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "t_after"), 1, "returns when the CHILD finishes");
    assert_eq!(
        u(&sim, "t_deep"),
        7,
        "the grandchild still runs to completion"
    );
}

/// All immediate children are still awaited — the fix must not weaken that.
#[test]
fn wait_fork_still_waits_for_every_child() {
    let src = r#"
module tb;
  int t_after;
  initial begin
    fork
      #3;
      #7;
      #5;
    join_none
    wait fork;
    t_after = $time;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "t_after"),
        7,
        "the slowest immediate child gates the wait"
    );
}
