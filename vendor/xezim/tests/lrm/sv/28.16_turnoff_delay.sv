// top: t28_16
`timescale 1ns/1ns
module t28_16;
  logic a, en; wire o3;
  bufif1 #(1, 2, 4) g3 (o3, a, en);
  initial begin a = 1; en = 1; #10 en = 0; #10 $finish; end
  always @(o3) $display("T|a|t=%0t o3=%b (turn-off delay 4: z expected at 14)", $time, o3);
endmodule
