//! IEEE 1800-2017 §16.12.6 / §16.12.2-3: a property `or` pending at the end of
//! simulation fails only when BOTH operands fail, not when either does. The
//! `and` failed when either operand fails; the conjunction through a weak
//! operand holds, so `not (strong or weak)` fails once and
//! `(not weak) or (1 |-> weak)` does not.
fn outputs(src: &str) -> Vec<String> {
    let sim = xezim::simulate(src, 100_000).unwrap();
    sim.output.iter().map(|o| o.message.clone()).collect()
}

const OR_AT_END: &str = r#"
module top;
  logic clk = 0, a = 1, b = 0;
  int f_or = 0, f_not_or = 0;
  p_or: assert property (@(posedge clk)
      (not weak(a ##1 b)) or (1'b1 |-> weak(a ##1 b))) else f_or++;
  p_not_or: assert property (@(posedge clk)
      not ((1'b1 |-> strong(a ##1 b)) or (1'b1 |-> weak(a ##1 b)))) else f_not_or++;
  initial begin
    #5 clk = 1;
    #5 clk = 0;
  end
  final begin
    if (f_or == 0 && f_not_or == 1) $display("PASS");
    else $display("FAIL f_or=%0d (want 0) f_not_or=%0d (want 1)", f_or, f_not_or);
  end
endmodule
"#;

#[test]
fn or_fails_at_end_only_when_both_operands_fail() {
    let msgs = outputs(OR_AT_END);
    assert!(
        msgs.iter().any(|m| m.starts_with("PASS")),
        "expected PASS, got {:?}",
        msgs
    );
}

/// The `and` fails when either operand fails (unchanged): a bare
/// `(strong && weak)` conjunction fails at end because the strong operand
/// fails, even though the weak operand holds. A resolved Pass on one `or`
/// side (here not covered) would likewise hold.
#[test]
fn and_still_fails_with_one_failed_operand() {
    let src = r#"
module top;
  logic clk = 0, a = 1, b = 0;
  int f_and = 0, f_not_and = 0;
  p_and: assert property (@(posedge clk)
      (1'b1 |-> strong(a ##1 b)) and (1'b1 |-> weak(a ##1 b))) else f_and++;
  p_not_and: assert property (@(posedge clk)
      not ((1'b1 |-> strong(a ##1 b)) and (1'b1 |-> weak(a ##1 b)))) else f_not_and++;
  initial begin #5 clk = 1; #5 clk = 0; end
  // The conjunction fails once (strong operand fails at end); its `not` then
  // holds, so the negated form never fails.
  final begin
    if (f_and == 1 && f_not_and == 0) $display("PASS");
    else $display("FAIL f_and=%0d (want 1) f_not_and=%0d (want 0)", f_and, f_not_and);
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
