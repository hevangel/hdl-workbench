// top: t16_9b
module t16_9b;
  bit clk, e; int v;
  always #5 clk = ~clk;
  initial begin
    repeat (2) @(negedge clk); v = 1; e = 1; @(negedge clk) v = 2; e = 0; @(negedge clk) v = 3; @(negedge clk) v = 4; e = 1; @(negedge clk) $finish;
  end
  always @(posedge clk) $display("T|p|t=%0t v=%0d past=%0d past_gated=%0d", $time, v, $past(v), $past(v, 1, e));
endmodule
