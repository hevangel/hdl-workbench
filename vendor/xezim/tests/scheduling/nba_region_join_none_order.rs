//! IEEE 1800-2017 §4.4.2/§4.5 region ordering for the NBA-region wait
//! (`uvm_wait_for_nba_region`: `nba <= next_nba; @(nba)`): the NBA region
//! commits only after ALL active-region work of the time slot has settled —
//! in particular, a `fork ... join_none` child scheduled at the current time
//! (spawned by a thread that then parked on `@(nba)`) must get its first
//! slice BEFORE the parked thread resumes, or the resume observes
//! pre-child state.
//!
//! UVM 1800.2 `uvm_phase_hopper::execute_phase` relies on this: it forks
//! `master_phase_process` (traverse -> phase task -> `raise_objection`) and
//! parks on `uvm_wait_for_nba_region`; the NBA-region resume that follows
//! checks `phase_done.get_objection_total(top)`. With the child still queued
//! the check saw 0, skipped the ALL_DROPPED wait and ran the READY_TO_END
//! traverse one iteration early (a whole cluster of jump-phasing
//! regressions). Reference-validated ordering.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// The `uvm_wait_for_nba_region` implementation (`nba <= next_nba; @(nba)`)
/// with a forked join_none child whose first statement is the "objection
/// raise": the parent must see the child's write after the @(nba) resume.
#[test]
fn nba_region_resume_after_join_none_child() {
    let src = r#"
module top;
  bit nba = 0;
  bit next_nba = 1;
  int total = 0;
  initial begin
    fork
      begin
        total = total + 1; // the "objection raise" (immediate, active region)
        #10;
      end
    join_none
    // uvm_wait_for_nba_region
    nba <= next_nba;
    @(nba);
    $display("PARENT total=%0d", total);
    if (total == 1) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"PARENT total=1".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}

/// UVM execute_phase shape through CLASS methods (the hopper forks
/// master_phase_process, which forks the phase task's worker) — the child
/// chain runs in class-method trampolines; every level must start before the
/// NBA-region resume.
#[test]
fn nba_region_resume_after_join_none_class_chain() {
    let src = r#"
class C;
  int total;
  function new(int t); total = t; endfunction
  task worker();        // phase task: raises the objection
    total = total + 1;
    #10;
  endtask
  task thread_a();      // master_phase_process: traverse -> task_phase fork
    fork worker(); join_none;
    #10;                // wait(0): stays alive
  endtask
endclass
module top;
  bit nba = 0;
  int next_nba = 0;
  int total = 0;
  C c;
  task process_phase(); // uvm_phase_hopper::execute_phase
    c = new(total);
    fork c.thread_a(); join_none; // fork master_phase_process (join_none)
    // uvm_wait_for_nba_region + objection check
    next_nba++;
    nba <= next_nba;
    @(nba);
    total = c.total;
    if (total > 0) $display("TAG_PASS");
    else $display("TAG_FAIL");
  endtask
  initial begin
    fork process_phase(); join_none;
    #50;
  end
endmodule
"#;
    let sim = simulate(src, 60).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}
