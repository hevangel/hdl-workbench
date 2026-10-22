//! §28.3: a gate delay written without parentheses is one delay value (a
//! number or an identifier), so the terminal list after it is not taken as
//! a call's arguments — `and #6 (q, a, b);`, `buf #D (o, i);`. Expected lines
//! are the reference simulator's.

use xezim::simulate;

#[test]
fn unparenthesized_delay_before_a_nameless_gate() {
    let src = r#"
module top;
  parameter D = 3;
  reg a, b;
  wire q1, q2;
  and #6 (q1, a, b);
  buf #D (q2, a);
  initial begin
    a = 0; b = 1;
    #10 a = 1;
  end
  always @(q1 or q2) $display("G|%0t q1=%b q2=%b", $time, q1, q2);
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with("G|"))
        .collect();
    assert_eq!(
        got,
        [
            "G|3 q1=x q2=0",
            "G|6 q1=0 q2=0",
            "G|13 q1=0 q2=1",
            "G|16 q1=1 q2=1"
        ]
    );
}
