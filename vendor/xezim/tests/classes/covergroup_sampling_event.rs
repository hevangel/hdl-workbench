//! §19.3 sampling events of a covergroup: an `iff` on the event
//! (`@(posedge clk iff en)`) used to be ignored, so every edge sampled; and
//! a named event triggered right after a process woke on a clock edge was
//! missed by the edge scan, so `@(ev)` sampled nothing. Every expected
//! number was cross-checked against the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect()
}

fn assert_line(lines: &[String], want: &str) {
    assert!(
        lines.iter().any(|l| l == want),
        "expected `{want}`, got {lines:?}"
    );
}

/// Four clock edges with `en` low and two with it high: the `iff` covergroup
/// samples twice (2 of 8 values). The named event fires twice.
#[test]
fn event_iff_guard_and_named_event() {
    let l = lines(
        r#"
module tb;
  logic clk = 0, en = 0;
  logic [2:0] n = 0;
  event done_ev;
  always #5 clk = ~clk;
  covergroup cg_iff @(posedge clk iff en);
    cp : coverpoint n;
  endgroup
  covergroup cg_ev @(done_ev);
    cp : coverpoint n;
  endgroup
  cg_iff a = new();
  cg_ev  b = new();
  initial begin
    repeat (4) @(negedge clk) n = n + 1;
    en = 1;
    repeat (2) @(negedge clk) n = n + 1;
    -> done_ev; #1 n = n + 1; -> done_ev; #1;
    $display("iff %0.2f ev %0.2f", a.get_inst_coverage(), b.get_inst_coverage());
    $finish;
  end
endmodule
"#,
    );
    assert_line(&l, "iff 25.00 ev 25.00");
}

/// A named event triggered right after `@(negedge clk)` wakes the process.
#[test]
fn named_event_after_clock_wakeup() {
    let l = lines(
        r#"
module tb;
  logic clk = 0;
  logic [1:0] x = 0;
  event ev;
  always #5 clk = ~clk;
  covergroup cg @(ev);
    cp : coverpoint x;
  endgroup
  cg c = new();
  initial begin
    @(negedge clk) x = 1; -> ev;
    @(negedge clk) x = 2; -> ev;
    @(negedge clk) x = 3; -> ev;
    #1 $display("ev %0.2f", c.get_inst_coverage());
    $finish;
  end
endmodule
"#,
    );
    assert_line(&l, "ev 75.00");
}

/// A named event samples at the trigger, with the values of that moment.
#[test]
fn named_event_samples_at_the_trigger() {
    let l = lines(
        r#"
module tb;
  logic [2:0] x = 0;
  event ev;
  covergroup cg @(ev);
    c1 : coverpoint x { bins b1 = {1}; }
    c5 : coverpoint x { bins b5 = {5}; }
    c2 : coverpoint x { bins b2 = {2}; }
    c3 : coverpoint x { bins b3 = {3}; }
  endgroup
  cg c = new();
  initial begin
    #1 x = 1; -> ev; x = 5;
    #1 x = 2; -> ev; #0 x = 3;
    #1 $display("b1 %0.0f b5 %0.0f b2 %0.0f b3 %0.0f", c.c1.get_inst_coverage(), c.c5.get_inst_coverage(), c.c2.get_inst_coverage(), c.c3.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "b1 100 b5 0 b2 100 b3 0");
}

/// A class covergroup sampled on an event of its object, with an `iff`.
#[test]
fn class_event_with_iff() {
    let l = lines(
        r#"
class mon;
  bit [2:0] n;
  bit en;
  event tick;
  covergroup cg @(tick iff en);
    cp : coverpoint n;
  endgroup
  function new(); cg = new(); endfunction
endclass
module tb;
  mon m;
  initial begin
    m = new();
    repeat (6) begin
      #1 m.n = m.n + 1; m.en = m.n[0]; -> m.tick;
    end
    #1 $display("cls %0.2f", m.cg.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "cls 37.50");
}
