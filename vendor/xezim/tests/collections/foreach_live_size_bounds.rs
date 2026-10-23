//! IEEE 1800-2017 §12.7.3: a `foreach`'s iteration set is fixed when the loop
//! starts.
//!
//! For a queue / dynamic array with a blocking body, the loop runs on the
//! async `ForeachTail` continuation and re-reads the collection's LIVE size on
//! each iteration instead of trusting the key count frozen at entry. That
//! exists for one reason: a collection SHRUNK while the body was suspended
//! must exit early rather than index past the end.
//!
//! Growth was not bounded the same way. A `push_back` in the body raised the
//! bound exactly as fast as `idx` advanced, so the loop never exhausted — it
//! ran until max simulation time and stopped there, with no diagnostic and no
//! indication the `foreach` had never terminated. Clamping the live size to
//! the entry count makes `ForeachTail` provably terminate while leaving the
//! shrink behaviour it was written for intact.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const GROWS_DURING_LOOP: &str = r#"
`timescale 1ns/1ps
module top;
  bit clk = 0;
  always #5 clk = ~clk;
  int q[$];
  initial begin
    q.push_back(0);
    foreach (q[i]) begin
      @(posedge clk);       // blocking -> ForeachTail path
      q.push_back(i);       // must NOT extend the loop
      $display("NOTE: visit %0d size=%0d", i, q.size());
    end
    $display("NOTE: done");
    $finish;
  end
endmodule
"#;

/// §12.7.3 — appending inside the body cannot extend the iteration set.
/// Before the clamp this ran 10,000 times and was cut off by max sim time.
#[test]
fn foreach_growth_during_iteration_does_not_extend_the_loop() {
    assert_eq!(
        notes(GROWS_DURING_LOOP),
        vec!["NOTE: visit 0 size=2", "NOTE: done"],
        "the entry set held one element, so the loop must run exactly once"
    );
}

const SHRINKS_DURING_LOOP: &str = r#"
`timescale 1ns/1ps
module top;
  bit clk = 0;
  always #5 clk = ~clk;
  int q[$];
  initial begin
    q = '{10, 11, 12, 13, 14};
    foreach (q[i]) begin
      @(posedge clk);
      if (i == 1) begin q.delete(4); q.delete(3); end   // 5 -> 3 mid-loop
      $display("NOTE: visit %0d size=%0d", i, q.size());
    end
    $display("NOTE: done");
    $finish;
  end
endmodule
"#;

/// The reason the live size is consulted at all: a collection shrunk while the
/// body was suspended exits early instead of indexing past the end. The clamp
/// must not regress this.
#[test]
fn foreach_shrink_during_iteration_still_exits_early() {
    assert_eq!(
        notes(SHRINKS_DURING_LOOP),
        vec![
            "NOTE: visit 0 size=5",
            "NOTE: visit 1 size=3",
            "NOTE: visit 2 size=3",
            "NOTE: done",
        ],
        "live size 3 must end the loop early, not run to the entry count of 5"
    );
}
