// IEEE 1800-2017 16.9.3: in an action block of an assertion, a sampled-value
// function is clocked by the assertion's leading clock, so $past(v) is the
// value v had at the previous tick.
module t;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [3:0] v = 0;
  initial begin
    repeat (4) begin @(negedge clk); v = v * 5 + 3; end   // 3, 2, 13, 4
    @(negedge clk);
    $finish;
  end
  a_act: assert property (@(posedge clk) 1'b1) $display("t=%0t v=%0d past(v)=%0d", $time, v, $past(v));
endmodule
