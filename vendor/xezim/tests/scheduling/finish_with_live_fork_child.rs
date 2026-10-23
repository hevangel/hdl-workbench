//! §20.2 + §9.3.2: `$finish` ends the simulation even while a
//! `fork ... join_none` child is alive, and a fork inside an edge block
//! spawns REAL concurrent processes.
//!
//! Two stacked defects, found via a Verilator test_regress test whose whole
//! purpose is this rule (its shape is reproduced here):
//! 1. The bytecode compiler flattened ParBlock like a SeqBlock, so an edge
//!    block executed a join_none child's infinite `#1` loop INLINE on its
//!    own stack — the design froze (in_edge_block never cleared, no always
//!    block could fire again) and even SIGTERM was swallowed, since the
//!    interrupt flag is polled at loop boundaries the hang never reached.
//! 2. The nested delay loop (`run_events_until`) never checked
//!    `self.finished`, so a `$finish` raised by any process it serviced
//!    kept simulating as long as others kept scheduling.
//! The reference simulator prints the finish line at cyc=10 and exits.

use xezim::simulate;

const SRC: &str = r#"
module top;
  bit clk = 0;
  int cyc = 0;
  initial forever #1 clk = ~clk;
  always @(posedge clk) begin
    cyc = cyc + 1;
    if (cyc >= 10) begin
      $display("NOTE: finishing at cyc=%0d", cyc);
      $finish;
    end
  end
  always @(posedge clk) begin
    fork begin
      while (cyc != 99) #1;   // never true: a live child at $finish time
    end join_none
  end
endmodule
"#;

#[test]
fn finish_terminates_with_live_join_none_child() {
    let sim = simulate(SRC, 1_000_000).expect("simulate failed");
    let notes: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect();
    assert_eq!(notes, ["NOTE: finishing at cyc=10"]);
}
