//! §4.4.2 — a clock generator is a process like any other: its toggle at
//! time T was scheduled at its PREVIOUS toggle, so the events due at T keep
//! FIFO order around it. A `#delay` queued before that point resumes ahead
//! of the toggle (reads the old clock value, catches this slot's edge); one
//! queued after it — a process the last edge woke, or a time-0 process that
//! follows an `initial` generator in source order — resumes after the
//! toggle and waits for the next edge.
//!
//! xezim fired every generator after all of the slot's `#delay` wakeups, so
//! an AVIP's `#10 aresetn = 0; @(posedge aclk); aresetn = 1;` (clock
//! `initial` first, both scheduled at time 0) released reset in the same
//! slot it asserted it, and a BFM waiting `@(negedge aresetn);
//! @(posedge aresetn);` hung forever.
//!
//! Every expectation below was cross-checked against the reference
//! simulator, for both the `initial ... forever` and the `always` clock.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

fn reset_tb(clock_first: bool) -> String {
    let clk = "  initial begin aclk = 1'b0; forever #10 aclk = ~aclk; end\n";
    let rst = "  initial begin\n    aresetn = 1'b1;\n    #10 aresetn = 1'b0;\n    @(posedge aclk);\n    aresetn = 1'b1;\n    $display(\"%0t released\", $time);\n  end\n";
    let (a, b) = if clock_first { (clk, rst) } else { (rst, clk) };
    format!("module top;\n  bit aclk;\n  bit aresetn;\n{a}{b}  initial #100 $finish;\nendmodule\n")
}

#[test]
fn initial_clock_before_reset_toggles_first() {
    let o = lines(&reset_tb(true));
    assert!(o.iter().any(|l| l == "30 released"), "{o:?}");
}

#[test]
fn reset_before_initial_clock_catches_same_slot_edge() {
    let o = lines(&reset_tb(false));
    assert!(o.iter().any(|l| l == "10 released"), "{o:?}");
}

// An `always #10 aclk = ~aclk;` generator is a time-0 process too: declared
// before the reset `initial`, it queues its first toggle first, so the toggle
// at 10 precedes the reset's `#10` wakeup and the reset waits for the edge at
// 30. xezim gave every `always` generator the LAST time-0 rank, so it
// released at 10 regardless of source order.
fn always_reset_tb(seed: bool, clock_first: bool) -> String {
    let clk = if seed {
        "  initial aclk = 1'b0;\n  always #10 aclk = ~aclk;\n"
    } else {
        "  always #10 aclk = ~aclk;\n"
    };
    let rst = "  initial begin\n    aresetn = 1'b1;\n    #10 aresetn = 1'b0;\n    @(posedge aclk);\n    aresetn = 1'b1;\n    $display(\"%0t released\", $time);\n  end\n";
    let (a, b) = if clock_first { (clk, rst) } else { (rst, clk) };
    format!("module top;\n  bit aclk;\n  bit aresetn;\n{a}{b}  initial #100 $finish;\nendmodule\n")
}

#[test]
fn always_clock_before_reset_toggles_first() {
    let o = lines(&always_reset_tb(false, true));
    assert!(o.iter().any(|l| l == "30 released"), "{o:?}");
    let o = lines(&always_reset_tb(true, true));
    assert!(o.iter().any(|l| l == "30 released"), "{o:?}");
}

#[test]
fn reset_before_always_clock_catches_same_slot_edge() {
    let o = lines(&always_reset_tb(false, false));
    assert!(o.iter().any(|l| l == "10 released"), "{o:?}");
}

fn later_toggles_tb(always_clock: bool) -> String {
    let clk = if always_clock {
        "  initial clk = 0;\n  always #10 clk = ~clk;\n"
    } else {
        "  initial begin clk = 1'b0; forever #10 clk = ~clk; end\n"
    };
    format!(
        r#"module top;
  bit clk;
{clk}  initial begin @(posedge clk); #20; @(posedge clk); $display("S3 %0t", $time); end
  initial begin @(negedge clk); #10; @(posedge clk); $display("S4 %0t", $time); end
  initial begin #20; #10; @(posedge clk); $display("S5 %0t", $time); end
  initial begin @(negedge clk); #10; $display("S6 clk=%b", clk); end
  initial begin #20; #10; $display("S7 clk=%b", clk); end
  initial begin #30; @(posedge clk); $display("S8 %0t", $time); end
  initial #200 $finish;
endmodule
"#
    )
}

fn check_later_toggles(o: &[String]) {
    let want = ["S7 clk=0", "S6 clk=1", "S5 30", "S3 30", "S8 30", "S4 50"];
    let got: Vec<&String> = o.iter().filter(|l| l.starts_with('S')).collect();
    assert_eq!(got, want.iter().collect::<Vec<_>>(), "{o:?}");
}

#[test]
fn edge_woken_delay_resumes_after_forever_clock_toggle() {
    check_later_toggles(&lines(&later_toggles_tb(false)));
}

#[test]
fn edge_woken_delay_resumes_after_always_clock_toggle() {
    check_later_toggles(&lines(&later_toggles_tb(true)));
}
