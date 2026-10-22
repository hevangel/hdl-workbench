//! §16.14.3 `cover sequence`: it used to be a parse error. Like `cover
//! property` it runs on the concurrent-assertion machinery, but it counts
//! every match of an attempt, where `cover property` counts an attempt's
//! first. Cross-checked against the reference simulator: the sequence
//! matches at 25 and 35, the property at 25.

use xezim::simulate;

#[test]
fn cover_sequence_counts_every_match() {
    let sim = simulate(
        r#"
module top;
  logic clk = 0, a = 0, b = 0;
  always #5 clk = ~clk;
  cs : cover sequence (@(posedge clk) a ##[1:2] b) $display("%0t seq match", $time);
  cp : cover property (@(posedge clk) a ##[1:2] b) $display("%0t prop match", $time);
  initial begin
    @(negedge clk) a = 1;
    @(negedge clk) begin a = 0; b = 1; end
    @(negedge clk) b = 1;
    @(negedge clk) b = 0;
    #20 $finish;
  end
endmodule
"#,
        1000,
    )
    .expect("simulate failed");
    let l: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    let matches: Vec<&String> = l.iter().filter(|m| m.contains("match")).collect();
    assert_eq!(
        matches,
        ["25 seq match", "25 prop match", "35 seq match"],
        "{l:?}"
    );
    assert_eq!(sim.assertion_pass_total(), 3);
}
