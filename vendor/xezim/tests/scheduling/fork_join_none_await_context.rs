//! A `fork ... join_none` child parked on `process::await()` must be kept
//! SUSPENDED — not misjudged as finished — so its process context (`this` +
//! task locals) survives the awaited process terminating and the child
//! resumes with the shared handle intact.
//!
//! UVM's sequencer `uvm_sequencer_param_base::m_safe_select_item` forks a
//! `join_none` re-arbitration child that parks on `selected_sequence_request
//! .process_id.await()` then re-reads the request handle. xezim's
//! `is_pid_suspended` did not count an `await_waiters` park as suspended, so
//! such a child could be reported FINISHED and its context dropped, and a
//! follow-on read of the handle dereferenced null (`'selected_sequence_request
//! .request_id' read through a null handle`). The fix counts an await-parked
//! process as suspended (same class as the mailbox/event/semaphore parks).
//! Validated byte-for-byte against the reference simulator.

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn assert_pass(sim: &xezim::compiler::Simulator, tag: &str) {
    let msgs = messages(sim);
    let pass = msgs.iter().any(|m| m.contains(&format!("{tag}_PASS")));
    assert!(pass, "expected {tag}_PASS in output\nfull output: {msgs:?}");
}

/// A task forks a child that parks on a producer process's `await()` while the
/// task returns. The producer terminates after `#3`; on resume the child must
/// still see the shared class handle it references.
const AWAIT_PARK: &str = r#"
class rec_c; int id; endclass
process producer_p;
rec_c sel;
function void spawn_producer();
  fork
    begin
      producer_p = process::self();
      #3;
    end
  join_none
endfunction
task safe_select_item();
  begin
    spawn_producer();
    wait (producer_p != null);
    fork
      begin
        producer_p.await();   // park here until the producer (t=3) finishes
        if (sel == null) $display("AWAIT_FAIL  child lost handle");
        else $display("AWAIT_PASS child sees id=%0d", sel.id);
      end
    join_none
  end
endtask
module top;
  initial begin
    sel = new();
    sel.id = 42;
    safe_select_item();
    #20;
    $finish;
  end
endmodule
"#;

#[test]
fn join_none_child_await_preserves_context() {
    let sim = simulate(AWAIT_PARK, 200).expect("simulate failed");
    assert_pass(&sim, "AWAIT");
}
