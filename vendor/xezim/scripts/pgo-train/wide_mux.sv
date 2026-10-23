module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic sel = 0; logic [213:0] a, b, y; int cyc = 0;
  always_comb y = sel ? a : b;
  always @(posedge clk) begin sel <= ~sel; a <= a + 214'd3; b <= b - 214'd1; cyc <= cyc + 1; end
  initial begin
    a = 214'h10; b = 214'hf000;
    repeat (20000) @(posedge clk);
    #1 $display("MUX %h %h %0d", y[63:0], y[213:150], cyc);
    $finish;
  end
endmodule
