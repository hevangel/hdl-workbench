//! Issue #510 (§16.12.12): `until` with a property operand used to keep every
//! candidate tick of an attempt, so a long-running attempt took quadratic time
//! (and overlapping attempts quadratic memory). Finished front ticks whose
//! left held and whose right has already failed are now folded into the
//! attempt's accumulated nonvacuity and dropped, bounding both. This test
//! checks the long-running attempt still resolves correctly (PASS at the end)
//! and, by using a large N that the quadratic scheduler would take several
//! seconds on, regression-guards the folding. Verified against a reference
//! simulator.
use xezim::simulate;

#[test]
fn until_property_operand_long_attempt_folds_ticks() {
    // `busy until (done ##1 ack)` with busy held for N cycles: the right
    // sequence stays pending (done=0) and, being `##1`, fails each sampled
    // tick. Every such tick is foldable, so the attempt's tick list must stay
    // O(1) regardless of N. The attempt succeeds with a nonvacuous pass at the
    // end when done ##1 ack completes.
    let src = r#"module top;
      localparam int N = 50000;
      logic clk = 0, start = 0, busy = 0, done = 0, ack = 0;
      int passes = 0, fails = 0;
      always #5 clk = ~clk;
      initial $assertvacuousoff;
      a_until: assert property (@(posedge clk) start |-> (busy until (done ##1 ack)))
        passes++; else fails++;
      initial begin
        @(negedge clk) begin start = 1; busy = 1; end
        @(negedge clk) start = 0;
        repeat (N) @(negedge clk);
        @(negedge clk) done = 1;
        @(negedge clk) begin done = 0; ack = 1; busy = 0; end
        @(negedge clk) ack = 0;
        @(negedge clk);
        if (passes == 1 && fails == 0) $display("T|PASS");
        else $display("T|FAIL %0d %0d", passes, fails);
        $finish;
      end
    endmodule"#;
    let result = simulate(src, 100_000_000).unwrap();
    let got: Vec<String> = result
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(got, vec!["T|PASS"], "long until attempt must pass");
}

#[test]
fn until_property_operand_fails_when_busy_drops_before_right() {
    // `busy until (done ##1 ack)` where busy drops while the right sequence is
    // still pending: the attempt must FAIL (left operand violated).
    let src = r#"module top;
      logic clk = 0, start = 0, busy = 0, done = 0, ack = 0;
      int passes = 0, fails = 0;
      always #5 clk = ~clk;
      initial $assertvacuousoff;
      a_until: assert property (@(posedge clk) start |-> (busy until (done ##1 ack)))
        passes++; else fails++;
      initial begin
        @(negedge clk) begin start = 1; busy = 1; end
        @(negedge clk) start = 0;
        @(negedge clk) busy = 0;                    // left drops while right pending
        @(negedge clk);
        @(negedge clk) done = 1;                    // too late
        @(negedge clk);
        if (passes == 0 && fails == 1) $display("T|PASS");
        else $display("T|FAIL %0d %0d", passes, fails);
        $finish;
      end
    endmodule"#;
    let result = simulate(src, 100_000).unwrap();
    let got: Vec<String> = result
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(got, vec!["T|PASS"], "until must fail when left drops early");
}