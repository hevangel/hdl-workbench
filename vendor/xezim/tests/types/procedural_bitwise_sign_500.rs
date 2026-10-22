//! IEEE 1800 §11.8.1: integral bitwise results retain operand signedness.

use xezim::simulate;

fn check(source: &str, expected: &[&str]) {
    let result = simulate(source, 100).unwrap();
    let actual: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn original_issue_500() {
    let result = simulate(include_str!("../issue_cases/issue500.sv"), 100).unwrap();
    assert!(
        result.output.iter().any(|o| o.message == "ALL PASS"),
        "{:?}",
        result.output
    );
    assert!(
        !result.output.iter().any(|o| o.message.starts_with("FAIL")),
        "{:?}",
        result.output
    );
}

#[test]
fn signed_bitwise_operators_and_xnor_keep_sign() {
    check(
        r#"module top;int a=-10,b=0,c=-1;
 initial begin
 $display("T|ops %0d %0d %0d %0d",(a&c)<0,(a|b)<0,(a^b)<0,(a~^c)<0);
 $display("T|extend %0d %0d %0d %0d",(a&c)+64'sd0,(a|b)+64'sd0,(a^b)+64'sd0,(a~^c)+64'sd0);
 end endmodule"#,
        &["T|ops 1 1 1 1", "T|extend -10 -10 -10 -10"],
    );
}

#[test]
fn an_unsigned_operand_makes_the_result_unsigned() {
    check(
        r#"module top;logic signed[7:0] a=-10;logic[7:0] b=8'hff;
 initial begin
 $display("T|mixed %0d %0d %0d %0d",(a&b)<0,(a|8'h00)<0,(a^8'h00)<0,(a~^b)<0);
 $display("T|values %0d %0d",(a&b)+64'sd0,(a&8'shff)+64'sd0);
 end endmodule"#,
        &["T|mixed 0 0 0 0", "T|values 246 -10"],
    );
}

#[test]
fn narrow_signed_operands_extend_before_bitwise_evaluation() {
    check(
        r#"module top;logic signed[7:0] a=-10;logic signed[15:0] b=-1;
 initial begin
 $display("T|width %0d %0d %0d",(a&b)+64'sd0,(a|16'sd0)+64'sd0,(a^16'sd0)+64'sd0);
 end endmodule"#,
        &["T|width -10 -10 -10"],
    );
}

#[test]
fn wide_bitwise_results_keep_sign() {
    check(
        r#"module top;logic signed[127:0] a=-10,b=0,c=-1;
 initial $display("T|wide %0d %0d %0d %0d",(a&c)<0,(a|b)<0,(a^b)<0,(a~^c)<0);
 endmodule"#,
        &["T|wide 1 1 1 1"],
    );
}

#[test]
fn unknown_sign_bits_propagate_through_arithmetic_shift() {
    check(
        r#"module top;logic signed[7:0] a=8'bx0000000,b=8'bz0000000;
 initial $display("T|xz %0d %0d %0d",((a|8'sh00)>>>1)===8'sbxx000000,((b^8'sh00)>>>1)===8'sbxx000000,((a&8'shff)>>>1)===8'sbxx000000);
 endmodule"#,
        &["T|xz 1 1 1"],
    );
}

#[test]
fn reductions_and_concatenations_remain_unsigned() {
    check(
        r#"module top;int a=-1;
 initial $display("T|self %0d %0d %0d %0d",(&a)<0,(|a)<0,(^a)<0,{a}<0);
 endmodule"#,
        &["T|self 0 0 0 0"],
    );
}

#[test]
fn function_local_and_argument_contexts_preserve_sign() {
    check(
        r#"module top;
 function automatic longint calc(int a);int z=0;return (a|z)+64'sd0;endfunction
 function automatic int negative(int a);return (a^0)<0;endfunction
 initial $display("T|functions %0d %0d",calc(-10),negative(-10));
 endmodule"#,
        &["T|functions -10 1"],
    );
}

#[test]
fn unsigned_comparison_context_remains_unsigned() {
    check(
        r#"module top;logic signed[7:0] a=8'bx0000000;
initial $display("T|context %0d",((a|8'sh00)>>>1)===8'b0x000000);
endmodule"#,
        &["T|context 1"],
    );
}
