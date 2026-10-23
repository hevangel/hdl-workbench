
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [7:0] mem [0:3];
  logic [7:0] acc = 0; logic [1:0] p = 0;
  always @(posedge clk) begin
    p <= p + 1;
    acc <= acc ^ mem[p];
  end
  initial begin
    repeat (20000) @(posedge clk);
    #1 $display("XB %b %0d", acc, p);
    $finish;
  end
endmodule
