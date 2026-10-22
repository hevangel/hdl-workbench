// top: rci
`timescale 1ns/1ns
module rci;
  logic clk = 0;
  always #5 clk = ~clk;
  wire [7:0] bidir;
  logic [7:0] gnt = 0;
  always @(posedge clk) gnt <= gnt + 1;
  clocking cb @(posedge clk);
    default input #1 output #1;
    inout bidir;
    input gnt;
  endclocking
  initial begin
    @(cb);
    $display("T|r1|t=%0t cb.gnt=%0d gnt=%0d", $time, cb.gnt, gnt);
    cb.bidir <= 8'h5a;
    #3 $display("T|r2|t=%0t bidir=%h", $time, bidir);
    @(cb);
    $display("T|r3|t=%0t cb.bidir=%h", $time, cb.bidir);
    $finish;
  end
endmodule
