module top;
  logic clk = 0;
  int n = 0;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    if (n == 2) $finish;
    n++;
  end
  final if (n == 2) $display("PASS n=%0d", n); else $display("FAIL n=%0d", n);
endmodule
