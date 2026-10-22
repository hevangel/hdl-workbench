module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic en = 0; logic [270:0] d, q; int cyc = 0;
  always_comb if (en) q = d; else q = {271{1'bx}};
  always @(posedge clk) begin en <= ~en; d <= d + 271'd5; cyc <= cyc + 1; end
  initial begin
    d = 271'h77;
    repeat (20000) @(posedge clk);
    #1 $display("XF %b %h %0d", q[270], q[15:0], cyc);
    $finish;
  end
endmodule
