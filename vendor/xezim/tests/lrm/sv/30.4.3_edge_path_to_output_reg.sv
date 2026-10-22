// top: t30_4
`timescale 1ns/1ps
module ffs (input clk, d, output reg q);
  always @(posedge clk) q <= d;
  specify (posedge clk => (q +: d)) = (3, 3); endspecify
endmodule
module ffw (input clk, d, output q);
  reg qr; always @(posedge clk) qr <= d; assign q = qr;
  specify (posedge clk => (q +: d)) = (3, 3); endspecify
endmodule
module t30_4;
  logic clk = 0, d = 0; wire q1, q2;
  ffs u1 (clk, d, q1); ffw u2 (clk, d, q2);
  initial begin #5 d = 1; #5 clk = 1; #10 $finish; end
  always @(q1) $display("T|a|t=%0t q1(output reg)=%b (expect t=13)", $realtime, q1);
  always @(q2) $display("T|b|t=%0t q2(output wire)=%b (expect t=13)", $realtime, q2);
endmodule
