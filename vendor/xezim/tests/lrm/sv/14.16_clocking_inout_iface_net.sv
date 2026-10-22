// top: rvd
`timescale 1ns/1ns
interface cif(input logic clk);
  logic [7:0] req;
  wire [7:0] bidir;
  clocking cb @(posedge clk);
    default input #1 output #1;
    output req;
    inout bidir;
  endclocking
endinterface
module rvd;
  logic clk = 0;
  always #5 clk = ~clk;
  cif ifc(clk);
  virtual cif vif;
  initial begin
    vif = ifc;
    @(vif.cb);
    vif.cb.req <= 8'h11;
    vif.cb.bidir <= 8'h5a;
    #3 $display("T|r1|t=%0t req=%h bidir=%h", $time, ifc.req, ifc.bidir);
    @(ifc.cb);
    ifc.cb.bidir <= 8'h77;
    #3 $display("T|r2|t=%0t bidir=%h", $time, ifc.bidir);
    $finish;
  end
endmodule
