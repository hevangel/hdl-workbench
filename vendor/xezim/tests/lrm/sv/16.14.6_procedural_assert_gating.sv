// top: t16_14
module t16_14;
  bit clk, start, done;
  always #5 clk = ~clk;
  initial begin
    @(negedge clk) start = 1;
    @(negedge clk) start = 0;
    @(negedge clk) done = 1;
    @(negedge clk) done = 0;
    repeat (4) @(negedge clk); $finish;
  end
  always @(posedge clk) begin
    if (start) a_inf: assert property (##2 done) $display("T|inf|P t=%0t", $time); else $display("T|inf|F t=%0t", $time);
  end
endmodule
