//! `wait(ev)` on a named event waits for a NEW trigger every time (§15.5.3):
//! the event is true only in the time slot it was triggered, like
//! `ev.triggered`. Its stored value stayed set after the first trigger, so a
//! second `wait(ev)` fell straight through. The reference simulator prints
//! 1100000 and 2200000.

#[test]
fn wait_on_a_named_event_waits_for_each_trigger() {
    let sim = xezim::simulate(
        r#"
`timescale 1ns/1ps
module top;
  event done_ev;
  bit req = 0;
  initial forever begin @(posedge req); req = 0; #100; -> done_ev; end
  initial begin
    #1000;
    req = 1; wait (done_ev); $display("W|first t=%0t", $time);
    #1000;
    req = 1; wait (done_ev); $display("W|second t=%0t", $time);
    $finish;
  end
endmodule
"#,
        5_000_000,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(o.iter().any(|l| l == "W|first t=1100000"), "{o:?}");
    assert!(o.iter().any(|l| l == "W|second t=2200000"), "{o:?}");
}
