//! §4.5 — a comb block triggered by a blocking write must observe the writing
//! process's FINAL state, not an intermediate one.
//!
//! xezim settles inline after every blocking assign, and that is load-bearing:
//! nets and primitives have to propagate immediately (deferring them wholesale
//! regresses UDP edge evaluation and flop wake-up). But it also meant a
//! triggered `always @(*)` ran in the MIDDLE of another process's statement
//! sequence — so a later statement in that same process could overwrite what
//! the block had just computed, and the block was never recomputed because its
//! own inputs had not changed:
//!
//!     initial begin a = 0; b = 1; g = 0; end     // g = 0 lands LAST
//!     always @(*) if (a ^ b) g = 8'h55;          // ran after `b = 1`
//!
//! left `g` at 0; a reference simulator yields 0x55.
//!
//! Rather than reorder the schedule, the entries that ran mid-process are
//! recorded with the outputs they produced and re-run at the process's exit
//! ONLY if those outputs were clobbered — so nothing else moves, and blocks
//! with side effects are not re-executed in the common case.
//!
//! Not covered (and deliberately so): two INDEPENDENT processes racing to write
//! the same variable at time 0. Their relative order is implementation-defined,
//! and a reference simulator rejects the same shape outright when the writer is
//! `always_comb` ("variable driven in a combinational block, may not be driven
//! by any other process").

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SRC: &str = r#"
module tb;
  logic a1, b1, a2, b2, a3, b3;
  logic [7:0] g1, g2, g3;
  integer n1, n2, n3;

  // The counter/flag is cleared AFTER the stimulus, in the same process.
  initial begin a1 = 1'b0; b1 = 1'b1; g1 = 0; n1 = 0; end
  always @(*) begin if (a1 ^ b1) g1 = 8'h55; end
  always @(*) begin if (a1 ^ b1) n1 = n1 + 1; end   // read-modify-write

  // Control: cleared BEFORE the stimulus — this always worked.
  initial begin g2 = 0; n2 = 0; a2 = 1'b0; b2 = 1'b1; end
  always @(*) begin if (a2 ^ b2) g2 = 8'h55; end
  always @(*) begin if (a2 ^ b2) n2 = n2 + 1; end

  // Delayed processes run outside the time-zero edge-dispatch context. A
  // replayed block may itself perform a blocking write that needs settling.
  initial begin #1; a3 = 1'b0; b3 = 1'b1; g3 = 0; n3 = 0; end
  always @(*) begin if (a3 ^ b3) g3 = 8'h55; end
  always @(*) begin if (a3 ^ b3) n3 = n3 + 1; end

  logic [7:0] seen_g1, seen_g2, seen_g3;
  integer seen_n1, seen_n2, seen_n3;
  initial begin
    #20;
    seen_g1 = g1;
    seen_g2 = g2;
    seen_g3 = g3;
    seen_n1 = n1;
    seen_n2 = n2;
    seen_n3 = n3;
  end
endmodule
"#;

#[test]
fn comb_output_overwritten_later_in_the_same_process_is_recomputed() {
    let sim = simulate(SRC, 200).expect("simulate failed");
    assert_eq!(get(&sim, "seen_g1") & 0xFF, 0x55);
    assert_eq!(get(&sim, "seen_g2") & 0xFF, 0x55);
    assert_eq!(get(&sim, "seen_g3") & 0xFF, 0x55);
    // Fires exactly once — a re-run must not turn into a self-retrigger loop.
    assert_eq!(get(&sim, "seen_n1"), 1);
    assert_eq!(get(&sim, "seen_n2"), 1);
    assert_eq!(get(&sim, "seen_n3"), 1);
}
