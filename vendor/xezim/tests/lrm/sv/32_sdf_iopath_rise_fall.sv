// top: c32_sdf   (needs 32_sdf_iopath_rise_fall.sdf in cwd; rise 5 / fall 8 typ)
`timescale 1ns/1ps
module bufc (input a, output y);
  buf b0 (y, a);
  specify (a => y) = 1; endspecify
endmodule
module c32_sdf;
  logic a; wire y;
  bufc u1 (a, y);
  initial begin
    $sdf_annotate("32_sdf_iopath_rise_fall.sdf", c32_sdf);
    a = 0; #20 a = 1; #20 a = 0; #20 $finish;
  end
  always @(y) $display("T|32|t=%0t y=%b", $realtime, y);
endmodule
