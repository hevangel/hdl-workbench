//! A fork child that outlives its creating method must not leak its *stale*
//! automatic-locals into the unrelated activation that later reuses the same
//! caller-frame slot.
//!
//! §6.21/§9.3.2: automatic variables of a method are per-activation. xezim
//! models these as call-frame slots, so two different task definitions with
//! the same local name (here `count`) occupy the *same* slot index when the
//! tasks are called from one caller. A `fork ... join_any` child (the `forever`
//! reader below) is still alive after its creating method returns; per the
//! standard its `count` — a copy of the *first* activation's automatic — is
//! dead, and its later writes must NOT land in the second activation's
//! `count` slot.
//!
//! Pre-fix, the orphaned reader from `alpha` kept merging its stale `count`
//! into the parent's current frame, which by then held `beta`'s `count`, so
//! `beta` finished with a corrupted count (alpha's orphan increments added
//! on top of beta's real ones). The fix tags each frame slot with a generation
//! and only lets a child merge into a slot that still belongs to the activation
//! it forked from (see `merge_fork_writes`).

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn assert_has(sim: &xezim::compiler::Simulator, needle: &str) {
    let msgs = messages(sim);
    assert!(
        msgs.iter().any(|m| m.contains(needle)),
        "expected output containing {needle:?}\nfull output: {msgs:?}"
    );
}

/// Two different task definitions each fork a `forever` reader that counts
/// written signal-events, plus a writer, with `join_any` on the writer. The
/// writer finishes first, the reader outlives the method. `beta` must report
/// its own count, not `plus alpha's orphan reader still incrementing`.
const ORPHAN_READER: &str = r#"
module top;
  event e;
  task alpha();
    automatic int count = 0;
    fork
      forever begin
        @(e);
        count++;
      end
      begin
        for (int i=0;i<3;i++) begin #1; ->e; end
        #1;
      end
    join_any
    $display("ALPHA=%0d", count);
  endtask
  task beta();
    automatic int count = 0;
    fork
      forever begin
        @(e);
        count++;
      end
      begin
        for (int i=0;i<5;i++) begin #2; ->e; end
        #1;
      end
    join_any
    $display("BETA=%0d", count);
  endtask
  initial begin
    alpha();
    beta();
    $finish;
  end
endmodule
"#;

#[test]
fn orphaned_fork_child_does_not_clobber_next_activation() {
    let sim = simulate(ORPHAN_READER, 400).expect("simulate failed");
    // alpha's writer fires 3 times -> alpha reads its own 3.
    assert_has(&sim, "ALPHA=3");
    // beta's writer fires 5 times -> beta must read exactly 5, NOT 8 (which
    // would mean alpha's still-running orphan reader merged its stale count).
    assert_has(&sim, "BETA=5");
    assert!(
        !messages(&sim).iter().any(|m| m.contains("BETA=8")),
        "beta's count must not absorb the orphaned alpha reader's increments: {:?}",
        messages(&sim)
    );
}
