// top: t20_12
module t20_12;
  bit clk, a; int fails;
  always #5 clk = ~clk;
  ap: assert property (@(posedge clk) a) else fails++;
  initial begin
    repeat (3) @(negedge clk); $display("T|a|base fails=%0d", fails);
    $assertoff(0, t20_12.ap); repeat (3) @(negedge clk); $display("T|b|after $assertoff fails=%0d (unchanged expected)", fails);
    $asserton(0, t20_12.ap); repeat (2) @(negedge clk); $display("T|c|after $asserton fails=%0d", fails);
    $assertcontrol(4, 15, 7, 0, t20_12.ap); repeat (2) @(negedge clk); $display("T|d|after $assertcontrol(off) fails=%0d", fails);
    $finish;
  end
endmodule
