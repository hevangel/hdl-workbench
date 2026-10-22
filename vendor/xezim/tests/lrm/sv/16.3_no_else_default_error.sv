// top: t16_3
module t16_3;
  bit clk; int x = 1;
  always #5 clk = ~clk;
  ap: assert property (@(posedge clk) x == 2);
  initial begin
    a1: assert (x == 2);
    assert (x == 3);
    #12 $display("T|a|done");
    $finish;
  end
endmodule
