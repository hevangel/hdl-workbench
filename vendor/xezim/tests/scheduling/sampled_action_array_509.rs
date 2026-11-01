//! IEEE 1800 §16.5.1 / §16.9.3: an unpacked-array element sampled in an
//! action block reads the Preponed (slot-entry) value, whether or not the
//! property reads that element. The array base has no "signal id", so the
//! element's cell must be routed through the select pipeline; and the runtime
//! `$sampled` must install that cell's Preponed value so an in-slot write to
//! the cell (a simultaneous NBA) does not defeat the sample.
use xezim::simulate;

/// Constant-index action sample: cell is toggled by a simultaneous NBA, so a
/// broken `$sampled(mem[0])` reads the live (post-NBA) 9 instead of the
/// Preponed 2.
#[test]
fn constant_index_action_sample_reads_preponed() {
    let src = r#"
module top;
  logic clk = 0;
  logic [3:0] mem [4] = '{4'd2, 4'd1, 4'd1, 4'd1};
  always #5 clk = ~clk;
  always_ff @(posedge clk) mem[0] <= 9;
  a: assert property (@(posedge clk) 1)
    begin
      $display("T|sample %0d live %0d", $sampled(mem[0]), mem[0]);
      $finish;
    end
endmodule
"#;
    let result = simulate(src, 20).expect("action array sampling must simulate");
    let line = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .find(|s| s.starts_with("T|sample"))
        .unwrap()
        .to_owned();
    assert_eq!(line, "T|sample 2 live 9");
}

/// A property that selects a cell by a variable index (`mem[i]`) while a
/// simultaneous NBA updates the toggling cell `mem[0]` that the action
/// samples: the action's `$sampled` must read the Preponed value of `mem[0]`
/// (its captured pre-edge value), matching the `pre` probe — it must not
/// drift to the live value the property's own variable-select installed.
#[test]
fn action_sample_when_property_has_var_select() {
    let src = r#"
module top;
  logic clk = 0;
  int i = 1;
  logic mem [4] = '{default: 1'b0};
  int bad = 0, n = 0;
  logic pre;
  always #5 clk = ~clk;
  always_ff @(posedge clk) begin mem[0] <= ~mem[0]; mem[1] <= ~mem[1]; end
  always @(negedge clk) pre = mem[0];
  a: assert property (@(posedge clk) mem[i])   // i=1; mem[1] toggles
    begin
      n++;
      if ($sampled(mem[0]) !== pre) bad = bad + 1;
    end
  initial begin repeat (8) @(negedge clk);
    $display("RESULT n=%0d bad=%0d", n, bad);
    $finish;
  end
endmodule
"#;
    let result = simulate(src, 100).expect("var-select property + action sample");
    let line = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .find(|s| s.starts_with("RESULT"))
        .unwrap()
        .to_owned();
    // Four passing attempts (mem[1] true); none of the sampled mem[0] values
    // drift from their captured pre-edge value.
    assert_eq!(line, "RESULT n=4 bad=0");
}