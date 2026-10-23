module top;
  logic clk = 0;
  int n = 0;
  always @(posedge clk) n++;
  final if (n == 2) $display("PASS n=%0d", n); else $display("FAIL n=%0d", n);
endmodule
