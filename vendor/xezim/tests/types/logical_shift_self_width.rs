//! IEEE 1800-2017 §11.4.10: the right operand of a shift is self-determined
//! and cannot widen the shift result when it appears under a logical operator.

use xezim::simulate;

#[test]
fn logical_or_preserves_the_shift_left_operand_width() {
    let sim = simulate(
        r#"
module shell;
  logic phase = 0; always #5 phase = ~phase;
  logic permit = 0;
  logic [1:0] field = 2'b11;
  integer hits = 0;
  initial if (permit || ((~field) >> 1)) hits++;
  always @(posedge phase) if (permit || ((~field) >> 1)) hits++;
  initial begin #6; $display("HITS=%0d", hits); $finish; end
endmodule
"#,
        100,
    )
    .expect("simulate");
    let text = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("HITS=0"),
        "compiled expression was widened:\n{text}"
    );
}
