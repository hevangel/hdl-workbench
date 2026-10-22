//! IEEE 1800 §16.5.1 / §16.9.3: action-only operands use Preponed values.
use xezim::simulate;

fn check(action: &str, fail: bool) {
    let property = if fail { "0) else" } else { "1)" };
    let src = format!(
        r#"
module top;
  logic clk = 0;
  logic [3:0] q = 2;
  always #5 clk = ~clk;
  always_ff @(posedge clk) q <= 9;
  assert property (@(posedge clk) {property} begin
    {action}
    $display("T|live %0d", q);
    $finish;
  end
endmodule
"#
    );
    let result = simulate(&src, 20).expect("action sampling must simulate");
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(lines, ["T|sample 2", "T|live 9"]);
}

#[test]
fn pass_action() {
    check(r#"$display("T|sample %0d", $sampled(q));"#, false);
}
#[test]
fn fail_action() {
    check(r#"$display("T|sample %0d", $sampled(q));"#, true);
}
#[test]
fn for_condition() {
    check(
        r#"int n; n = 0; for (int k=0; k<$sampled(q); k++) n++; $display("T|sample %0d", n);"#,
        false,
    );
}
#[test]
fn repeat_count() {
    check(
        r#"int n; n = 0; repeat ($sampled(q)) n++; $display("T|sample %0d", n);"#,
        false,
    );
}
#[test]
fn automatic_initializer() {
    check(
        r#"automatic int n = int'($sampled(q)); $display("T|sample %0d", n);"#,
        false,
    );
}
#[test]
fn while_condition() {
    check(
        r#"int n; n = 0; while (n<$sampled(q)) n++; $display("T|sample %0d", n);"#,
        false,
    );
}
#[test]
fn case_item() {
    check(
        r#"case (2) $sampled(q): $display("T|sample 2"); default: $display("T|wrong"); endcase"#,
        false,
    );
}
