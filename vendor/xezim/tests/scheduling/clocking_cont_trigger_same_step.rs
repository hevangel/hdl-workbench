//! A process resumed by a clocking-block event (`##2`, `@(cb)`) runs after
//! the tick's edge pass. A named event or signal it writes had no pass left
//! in that tick to wake `@(ev)` / `@(sig)` waiters, which resumed one clock
//! toggle later: `##2; -> ev;` was seen by a waiter parked on the event at
//! the NEXT toggle, so a test comparing `$realtime` on both sides failed.
//! Expected times are those of the reference simulator.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn trigger_from_a_clocking_resumed_process_wakes_waiters_in_the_same_step() {
    let msgs = messages(
        "`timescale 1ns/1ps
module tb;
  reg clk = 0; always #5 clk = ~clk;
  default clocking cb @(posedge clk); default input #1ps output #1ps; endclocking
  event ev; logic flag = 0;
  initial begin
    #150;
    fork
      begin ##2; $display(\"%0t A trigger\", $realtime); -> ev; flag = 1; end
      begin @(cb); @(ev); $display(\"%0t B woke on ev\", $realtime); end
      begin @(cb); @(posedge flag); $display(\"%0t C woke on flag\", $realtime); end
    join
    $display(\"%0t joined\", $realtime);
    $finish;
  end
endmodule",
    );
    for want in [
        "165000 A trigger",
        "165000 B woke on ev",
        "165000 C woke on flag",
        "165000 joined",
    ] {
        assert!(msgs.iter().any(|m| m == want), "missing {want}: {msgs:?}");
    }
}
