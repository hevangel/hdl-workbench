module top;
  logic clk = 0;
  int q = 0, n = 0;
  always #5 clk = ~clk;
  always @(posedge clk) q <= q + 1;
  always @(posedge clk) begin n++; if (n == 3) $finish; end
  final if (q == 2) $display("PASS q=%0d", q); else $display("FAIL q=%0d", q);
endmodule
