//! §21.2.1.7: `%p` of a `string` parameter prints the quoted text, as it
//! does for a string variable; it printed the packed bytes as a number.
//! Cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn p_format_quotes_string_parameters() {
    let src = r#"
module tb;
  parameter string SP = "hello";
  localparam string LP = "wor";
  parameter int IP = 5;
  initial $display("p=%p %p %p", SP, LP, IP);
endmodule
"#;
    let sim = simulate(src, 1_000).expect("simulate failed");
    let out: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(out, [r#"p="hello" "wor" 5"#]);
}
