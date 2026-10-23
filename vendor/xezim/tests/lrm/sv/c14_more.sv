// top: c14x
`timescale 1ns/1ns
interface cif(input logic clk);
  logic [7:0] req, gnt;
  wire [7:0] bidir;
  clocking cb @(posedge clk);
    default input #1 output #1;
    output req;
    input gnt;
    inout bidir;
  endclocking
  modport tb (clocking cb);
endinterface
class drv_c;
  virtual cif vif;
  function new(virtual cif v); vif = v; endfunction
  task run();
    @(vif.cb);
    vif.cb.req <= 8'h11;
    @(vif.cb);
    $display("T|14.9a|t=%0t req=%h gnt_sampled=%h", $time, vif.req, vif.cb.gnt);
    vif.cb.bidir <= 8'h5a;
    @(vif.cb);
    #2 $display("T|14.9b|t=%0t bidir=%h cb.bidir=%h", $time, vif.bidir, vif.cb.bidir);
  endtask
endclass
module c14x;
  logic clk = 0;
  always #5 clk = ~clk;
  cif ifc(clk);
  always @(posedge clk) ifc.gnt <= ifc.req + 1;
  // 14.16.2 multiple drives to the same clockvar in the same cycle: last wins
  logic [7:0] tgt;
  clocking cb2 @(posedge clk); output #0 tgt; endclocking
  // 14.10 clocking block signal event @(cb.sig)
  logic [3:0] ev_sig = 0;
  clocking cb3 @(posedge clk); input ev_sig; endclocking
  initial begin
    drv_c d = new(ifc);
    ifc.req = 0;
    d.run();
    // last drive wins
    @(cb2); cb2.tgt <= 8'h01; cb2.tgt <= 8'h02;
    @(cb2); $display("T|14.16.2|tgt=%h", tgt);
    // 14.10: event on clockvar
    fork
      begin @(cb3.ev_sig); $display("T|14.10|cb3.ev_sig changed t=%0t val=%0d", $time, cb3.ev_sig); end
      begin #12 ev_sig = 4'h3; end
    join
    // 14.11 ##0 when not at a clocking event waits for next edge? (default clocking needed) -> use cb2
    #1 $finish;
  end
endmodule
