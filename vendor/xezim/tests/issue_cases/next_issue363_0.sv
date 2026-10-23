// IEEE 1800-2017 16.9.3: $past(expression1, number_of_ticks, expression2, clocking_event)
module t;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [3:0] v = 0, v1 = 0;
  int ticks = 0, fails = 0;
  always @(posedge clk) begin ticks <= ticks + 1; v1 <= v; end
  initial begin
    repeat (6) begin @(negedge clk); v = v * 5 + 3; end
    @(negedge clk);
    if (fails == 0) $display("PASS"); else $display("FAIL %0d", fails);
    $finish;
  end
  // v1 holds v one tick back.
  a_past: assert property (@(posedge clk) ticks < 1 || $past(v, 1, 1'b1, @(posedge clk)) == v1) else fails++;
endmodule
