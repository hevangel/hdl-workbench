//! Issue #508 (§16.14.8): `or` of a failing implication and a vacuous one is a
//! *nonvacuous* success, so with `$assertvacuousoff` the pass action fires.
//! The old code returned `Vacuous` whenever neither operand passed but one was
//! vacuous, so no pass action ran. Verified byte-for-byte against a reference
//! simulator: pass actions at ticks 15, 25 and 35.
use xezim::simulate;

#[test]
fn or_failing_impl_and_vacuous_is_nonvacuous_pass() {
    let src = r#"module top;
  logic clk = 0, a = 0, b = 0, c = 0, d = 0;
  int n_or = 0;
  always #5 clk = ~clk;  // rising edges at 5, 15, 25, 35
  initial $assertvacuousoff;
  a_or: assert property (@(posedge clk) (a |-> b) or (c |-> d))
    begin n_or++; $display("T_or|P %0t", $time); end
  initial begin
    @(negedge clk) begin a = 1; b = 0; end         // edge 15: a|->b fails, c|->d vacuous
    @(negedge clk) begin a = 0; c = 1; d = 0; end  // edge 25: a|->b vacuous, c|->d fails
    @(negedge clk) begin a = 1; b = 1; c = 0; end  // edge 35: a|->b holds,  c|->d vacuous
    @(negedge clk);
    $display("T_tot|P %0d", n_or);
    $finish;
  end
 endmodule"#;
    let result = simulate(src, 1000).unwrap();
    let got: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T_"))
        .collect();
    assert_eq!(
        got,
        vec![
            "T_or|P 15",
            "T_or|P 25",
            "T_or|P 35",
            "T_tot|P 3",
        ]
    );
}