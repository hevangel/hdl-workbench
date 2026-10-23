module t;
  logic clk = 0;
  logic [7:0] src [0:1];
  logic [7:0] a [0:1], b [0:1], c [0:1], d [0:1];
  always @(posedge clk) a <= src;         // whole-array NBA in an always block
  always @(posedge clk) b = src;          // the blocking form, for comparison
  initial begin
    src[0] = 8'd1; src[1] = 8'd2;
    #1 clk = 1;
    c <= src;                             // whole-array NBA in an initial block
    d <= '{8'd3, 8'd4};                   // from an assignment pattern
    #1;
    $display("%s a = %0d %0d", (a[0] === 8'd1 && a[1] === 8'd2) ? "PASS" : "FAIL", a[0], a[1]);
    $display("%s b = %0d %0d", (b[0] === 8'd1 && b[1] === 8'd2) ? "PASS" : "FAIL", b[0], b[1]);
    $display("%s c = %0d %0d", (c[0] === 8'd1 && c[1] === 8'd2) ? "PASS" : "FAIL", c[0], c[1]);
    $display("%s d = %0d %0d", (d[0] === 8'd3 && d[1] === 8'd4) ? "PASS" : "FAIL", d[0], d[1]);
    $finish;
  end
endmodule
