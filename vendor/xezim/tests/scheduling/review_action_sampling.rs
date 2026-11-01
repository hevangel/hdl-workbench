//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    xezim::simulate(src, 1_000_000)
        .expect("simulation failed")
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn sampled_variable_index_uses_preponed_index() {
    let got = output(
        r###"module top;
 bit clk=0; int idx=0; int mem[2]='{2,3};
 always #5 clk=~clk;
 always @(posedge clk) begin idx <= 1; mem[0]<=9; mem[1]<=8; end
 a: assert property (@(posedge clk) 1) begin
  $display("T|sample %0d index %0d live %0d",$sampled(mem[idx]),$sampled(idx),mem[idx]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|sample 2 index 0 live 8"), "{got}");
}

#[test]
fn sampled_variable_index_expression() {
    let got = output(
        r###"module top;
 bit clk=0; int idx=0; int mem[2]='{2,3};
 always #5 clk=~clk;
 always @(posedge clk) begin idx <= 1; mem[0]<=9; mem[1]<=8; end
 a: assert property (@(posedge clk) 1) begin
  $display("T|sample %0d index %0d live %0d",$sampled(mem[idx ^ 0]),$sampled(idx),mem[idx ^ 0]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|sample 2 index 0 live 8"), "{got}");
}

#[test]
fn sampled_nested_array_index() {
    let got = output(
        r###"module top;
 bit clk=0; int idx=0; int mem[2]='{2,3};
 always #5 clk=~clk;
 always @(posedge clk) begin idx <= 1; mem[0]<=9; mem[1]<=8; end
 a: assert property (@(posedge clk) 1) begin
  $display("T|sample %0d index %0d live %0d",$sampled(mem[$sampled(idx)]),$sampled(idx),mem[idx]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|sample 2 index 0 live 8"), "{got}");
}
