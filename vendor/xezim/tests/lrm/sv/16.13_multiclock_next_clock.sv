// top: t16_13
module t16_13;
  bit c1, c2; bit a, b;
  always #5 c1 = ~c1;
  always #7 c2 = ~c2;
  initial begin a = 0; b = 0; #12 a = 1; #10 a = 0; #3 b = 1; #20 b = 0; #20 $finish; end
  mc1: assert property (@(posedge c1) a |=> @(posedge c2) b) $display("T|mc1|P t=%0t", $time); else $display("T|mc1|F t=%0t", $time);
  mc2: assert property (@(posedge c1) a ##1 @(posedge c2) b) $display("T|mc2|P t=%0t", $time); else $display("T|mc2|F t=%0t", $time);
endmodule
