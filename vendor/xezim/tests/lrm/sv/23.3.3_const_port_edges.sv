// top: rcp2
module edges2(input clk, output int pe, output int ne);
  int pcount = 0, ncount = 0;
  always @(posedge clk) begin pcount++; $display("T|r0|%m posedge at t=%0t clk=%b", $time, clk); end
  always @(negedge clk) ncount++;
  assign pe = pcount;
  assign ne = ncount;
endmodule
module rcp2;
  int pe0, ne0, pe1, ne1;
  edges2 e0(.clk(1'b0), .pe(pe0), .ne(ne0));
  edges2 e1(.clk(1'b1), .pe(pe1), .ne(ne1));
  initial #1 $display("T|r1|tie0: pe=%0d ne=%0d  tie1: pe=%0d ne=%0d", pe0, ne0, pe1, ne1);
endmodule
