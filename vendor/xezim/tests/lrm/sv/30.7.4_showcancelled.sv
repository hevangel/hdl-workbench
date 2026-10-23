// top: t30_7
`timescale 1ns/1ps
module pd (input a, output y3);
  buf (y3, a);
  specify
    showcancelled y3;
    pulsestyle_ondetect y3;
    (a => y3) = (5, 3);
  endspecify
endmodule
module t30_7;
  logic a; wire y3;
  pd u (a, y3);
  initial begin a = 0; #20 a = 1; #1 a = 0; #20 a = 1; #3 a = 0; #30 $finish; end
  always @(y3) $display("T|a|t=%0t y3=%b", $realtime, y3);
endmodule
