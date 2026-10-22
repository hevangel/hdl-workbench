// top: rrw
module rrw;
  logic clk = 0;
  time tt;
  int seen;
  task automatic refwait(ref logic sig, output time t); @(posedge sig); t = $time; endtask
  task automatic refwait2(ref logic sig); wait (sig == 1); seen = $time; endtask
  initial begin
    fork refwait(clk, tt); #4 clk = 1; join
    $display("T|r1|tt=%0t now=%0t", tt, $time);
    clk = 0;
    fork refwait2(clk); #3 clk = 1; join
    $display("T|r2|seen=%0d", seen);
  end
endmodule
