// top: c17_more
checker inner(logic x, event ck);
  a_in: assert property (@ck x |=> !x) else $display("T|17.3.2|inner F t=%0t", $time);
endchecker
checker outer(logic x, logic y, event ck);
  function automatic int dbl(int v); return 2 * v; endfunction
  bit [3:0] cnt = 0;
  always_ff @ck cnt <= cnt + x;
  covergroup cg @ck; coverpoint y; endgroup
  cg cgi = new;
  inner u_in(x, ck);
  c1: cover property (@ck dbl(cnt) == 4) $display("T|17.8|fn cover t=%0t cnt=%0d", $time, cnt);
  final $display("T|17.6|cg cov=%0.2f cnt=%0d", cgi.get_coverage(), cnt);
endchecker
module c17_more;
  bit clk; logic x, y;
  always #5 clk = ~clk;
  initial begin x = 0; y = 0; @(negedge clk) x = 1; @(negedge clk) x = 1; y = 1; @(negedge clk) x = 0; repeat (3) @(negedge clk); $finish; end
  outer u(x, y, posedge clk);
endmodule
