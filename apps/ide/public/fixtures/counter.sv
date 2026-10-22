`timescale 1ns/1ps
module counter;
  logic clk;
  logic rst;
  logic [3:0] count;

  initial begin
    clk = 1'b0;
    rst = 1'b1;
    #1 rst = 1'b0;
    repeat (8) begin
      #5 clk = ~clk;
    end
    #1 $display("COUNT=%0d", count);
    $finish;
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) count <= 4'd0;
    else count <= count + 1'b1;
  end
endmodule
