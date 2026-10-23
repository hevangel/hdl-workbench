//! IEEE 1800-2023 §16.9.3 / §16.5.1: `$sampled(e)` returns the sampled value
//! of `e` in the current time slot — its Preponed-region value — not its live
//! value after a same-slot update. The difference shows in a concurrent
//! assertion's action block, which runs after the NBA region.
use xezim::simulate;
fn outputs(source: &str) -> Vec<String> {
    let sim = simulate(source, 100_000).unwrap();
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// #421 original: at the posedge q is sampled as 0; the flop sets it to 1 in
/// the NBA before the pass action runs, so `$sampled(q)` in the action must
/// read 0. The reference simulators print PASS.
#[test]
fn sampled_reads_preponed_value_in_action_block() {
    let src = r#"
module t;
  logic clk = 0, q = 0, d = 1;
  always #5 clk = ~clk;
  always_ff @(posedge clk) q <= d;
  a: assert property (@(posedge clk) q == 0) begin
       if ($sampled(q) === 1'b0) $display("PASS");
       else $display("FAIL: $sampled(q)=%b", $sampled(q));
       $finish;
     end
endmodule
"#;
    let msgs = outputs(src);
    assert!(
        msgs.iter().any(|m| m.starts_with("PASS")),
        "expected PASS, got {:?}",
        msgs
    );
}

/// `$sampled` on a complex operand uses the Preponed value of its parts,
/// and does not disturb the live signal visible to the rest of the action
/// block.
#[test]
fn sampled_uses_preponed_parts_without_disturbing_live_reads() {
    let src = r#"
module t;
  logic clk = 0, q = 0, d = 1;
  always #5 clk = ~clk;
  always_ff @(posedge clk) q <= d;
  a: assert property (@(posedge clk) q == 0) begin
       // sampled q is 0, so the concatenation is 2'b01 (u is still 0).
       if ($sampled({q, 1'b0}) === 2'b00) $display("PASS");
       else $display("FAIL sampled=%b", $sampled({q, 1'b0}));
       // the live q (post-NBA) is 1 to the rest of the action block.
       if (q === 1'b1) $display("PASS");
       else $display("FAIL live q=%b", q);
       $finish;
     end
endmodule
"#;
    let msgs = outputs(src);
    assert_eq!(
        msgs.iter().filter(|m| m.starts_with("PASS")).count(),
        2,
        "expected two PASS, got {:?}",
        msgs
    );
}

/// $sampled outside an assertion (no active site) reads the live value.
#[test]
fn sampled_without_assertion_site_reads_live_value() {
    let src = r#"
module t;
  logic x = 0;
  initial begin
    #1 x = 1;
    #1 $display($sampled(x) === 1'b1 ? "PASS" : "FAIL live=%b", $sampled(x));
    $finish;
  end
endmodule
"#;
    let msgs = outputs(src);
    assert!(
        msgs.iter().any(|m| m.starts_with("PASS")),
        "expected PASS, got {:?}",
        msgs
    );
}
