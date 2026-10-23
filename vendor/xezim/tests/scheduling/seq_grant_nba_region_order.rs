//! IEEE 1800-2023 §4.4/§4.7 (event scheduling regions): after a parked
//! level-sensitive `wait()` process is GRANTED (its watched variable is
//! written in a late region), one NBA-region hop
//! (`nba <= next_nba; @(nba)` — the `uvm_wait_for_nba_region` idiom) must
//! let the granted process run its production code before the granting
//! side resumes to peek at the result.
//!
//! REGRESSION: a sequence-lib handshake (`wait_for_grant` /
//! `m_wait_for_arbitration_completed` shape) TIMEOUTed: the granted
//! process parked in `wait(lock_size != 0)` was resumable, but processes
//! parked across the NBA region during the end-of-tick late stages were
//! never re-drained in the same timestamp — the grant, the item PUT and
//! the peek all collapsed into the wrong delta and the handshake hung.
//! The late-region re-pass loop now drains condition waiters again after
//! the NBA region within the same time slot (reference-verified shape).
//! Reference simulator prints TAG_PASS / TAG_PASS2 for both checks.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

const SV_SRC: &str = r#"
module top;
  // uvm_wait_for_nba_region idiom
  int nba, next_nba;
  task automatic wait_nba();
    next_nba++;
    nba <= next_nba;
    @(nba);
  endtask

  int lock_size;    // watched by the parked sequence (m_lock_arb_size)
  int fifo_count;   // the "item put in m_req_fifo"

  // try_next_item shape: settle, grant, one NBA to produce, peek
  task automatic try_next();
    wait_nba();          // allow state to settle / choose next request
    lock_size++;         // m_set_arbitration_completed + m_update_lists
    wait_nba();          // "give it one NBA to put a new item in the fifo"
    if (fifo_count == 1)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL fifo_count=%0d", fifo_count);
  endtask

  // parked sequence shape: wait_for_grant -> m_wait_for_arbitration_completed
  task automatic seq_body();
    wait (lock_size != 0);
    fifo_count++;        // finish_item -> send_request -> m_req_fifo.put
  endtask

  initial begin
    fork
      try_next();
      seq_body();
    join
    if (fifo_count == 1) $display("TAG_PASS2"); else $display("TAG_FAIL2");
    $finish;
  end
endmodule
"#;

#[test]
fn grant_handshake_completes_within_one_nba_hop() {
    let sim = simulate(SV_SRC, 100).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS2".to_string()), "{o:?}");
    assert!(!o.iter().any(|l| l.starts_with("TAG_FAIL")), "{o:?}");
}
