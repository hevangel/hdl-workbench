module top;
  reg clk = 0;
  reg [7:0] o2;
  task stat;
    #1;
  endtask
  always @(posedge clk) begin : b2
    reg [7:0] other;
    if (other === 8'bx) other = 0;
    other = other + 1;
    o2 = other;
  end
  initial begin
    stat;
    repeat (3) begin #1 clk = 1; #1 clk = 0; end
    if (o2 === 8'h03) $display("PASS"); else $display("FAIL: o2 = %h", o2);
  end
endmodule
