// top: c30_spec
`timescale 1ns/1ps
module dut (input a, b, clk, d, sel, output y, z, q, w);
  assign y = a & b;
  assign z = ~a;
  reg qr; always @(posedge clk) qr <= d; assign q = qr;
  assign w = sel ? a : b;
  specify
    specparam tRise = 2, tFall = 3;
    (a => y) = (tRise, tFall);
    (b => y) = 4;
    (a *> z) = (1.5, 2.5);
    (posedge clk => (q +: d)) = (5, 6);
    if (sel) (a => w) = 7;
    if (!sel) (b => w) = 8;
    ifnone (a => w) = 1;
  endspecify
endmodule
module c30_spec;
  logic a, b, clk, d, sel; wire y, z, q, w;
  dut u (.*);
  initial begin
    a = 0; b = 1; clk = 0; d = 0; sel = 1;
    #20 a = 1; #20 a = 0; #20 b = 0; #20 b = 1; a = 1;
    #20 d = 1; #5 clk = 1; #20 clk = 0; d = 0; #5 clk = 1;
    #20 sel = 0; #20 b = 0; #20 b = 1;
    #20 sel = 1; a = 0; #20 a = 1;
    #50 $finish;
  end
  always @(y) $display("T|30.4|t=%0t y=%b", $realtime, y);
  always @(z) $display("T|30.4|t=%0t z=%b", $realtime, z);
  always @(q) $display("T|30.4.3|t=%0t q=%b", $realtime, q);
  always @(w) $display("T|30.4.4|t=%0t w=%b", $realtime, w);
endmodule
