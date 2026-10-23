// top: t24a
interface pif(input logic clk);
  logic [7:0] d;
  clocking cb @(posedge clk); default input #1step output #2; input d; output req = d; endclocking
  logic req;
endinterface
program automatic pg(input logic clk, ref int mod_cnt, pif p, output logic [3:0] po);
  int pv = 3;
  function int twice(int x); return 2 * x; endfunction
  initial begin
    @(posedge clk);
    // the design's always_ff NBA has already updated mod_q at this point
    $display("T|24.3a|t=%0t q=%0d cnt=%0d", $time, t24a.mod_q, mod_cnt);
    po = 4'h7;                      // blocking write from the program
    @(posedge clk);
    $display("T|24.3b|t=%0t samp=%0d seen_by_ff=%0d", $time, t24a.mod_q, t24a.ff_saw);
    @(p.cb);
    $display("T|24.3c|t=%0t cb.d=%0d d=%0d", $time, p.cb.d, p.d);
    p.cb.req <= 1;
    pv = twice(pv);
    #0 $display("T|24.3d|pv=%0d", pv);
  end
endprogram
module t24a;
  logic clk = 0;
  always #5 clk = ~clk;
  int mod_q = 0, ff_saw = -1, mod_cnt = 0;
  logic [3:0] po;
  pif pi(clk);
  always_ff @(posedge clk) mod_q <= mod_q + 1;
  always @(posedge clk) begin mod_cnt = mod_cnt + 1; ff_saw = po; end
  always @(posedge clk) pi.d <= mod_q * 10;
  pg p(.clk(clk), .mod_cnt(mod_cnt), .p(pi), .po(po));
  initial #60 $finish;
endmodule
