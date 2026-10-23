// §16.12.2–3: weak and strong pending operands compose under property not,
// and, or and implication when no further clock ticks arrive.
module top;
  logic clk = 0, a = 1, b = 0, reset = 0;
  int failures[19];
  p0: assert property (@(posedge clk) not (a ##1 b)) else failures[0]++;
  p1: assert property (@(posedge clk) not weak(a ##1 b)) else failures[1]++;
  p2: assert property (@(posedge clk) not strong(a ##1 b)) else failures[2]++;
  p3: assert property (@(posedge clk) not not (a ##1 b)) else failures[3]++;
  p4: assert property (@(posedge clk) not not strong(a ##1 b)) else failures[4]++;
  p5: assert property (@(posedge clk) not not not (a ##1 b)) else failures[5]++;
  p6: assert property (@(posedge clk) 1'b1 |-> not (a ##1 b)) else failures[6]++;
  p7: assert property (@(posedge clk) 1'b1 |=> not (a ##1 b)) else failures[7]++;
  p8: assert property (@(posedge clk) not (1'b1 |-> strong(a ##1 b))) else failures[8]++;
  p9: assert property (@(posedge clk)
      not ((1'b1 |-> strong(a ##1 b)) or (1'b1 |-> weak(a ##1 b)))) else failures[9]++;
  p10: assert property (@(posedge clk)
      (not weak(a ##1 b)) or (1'b1 |-> weak(a ##1 b))) else failures[10]++;
  p11: assert property (@(posedge clk)
      not ((1'b1 |-> strong(a ##1 b)) and (1'b1 |-> weak(a ##1 b)))) else failures[11]++;
  p12: assert property (@(posedge clk) not ((a ##1 b) or (a ##2 b))) else failures[12]++;
  p13: assert property (@(posedge clk) not strong((a ##1 b) or (a ##2 b))) else failures[13]++;
  p14: assert property (@(posedge clk)
      (not weak(a ##1 b)) and (1'b1 |-> weak(a ##1 b))) else failures[14]++;
  p15: assert property (@(posedge clk)
      not ((1'b1 |-> 1'b0) or (1'b1 |-> weak(a ##1 b)))) else failures[15]++;
  p16: assert property (@(posedge clk)
      not ((1'b1 |-> 1'b1) and (1'b1 |-> strong(a ##1 b)))) else failures[16]++;
  p17: assert property (@(posedge clk) not (1'b0 ##1 b)) else failures[17]++;
  p18: assert property (@(posedge clk) disable iff (reset) not (a ##1 b)) else failures[18]++;
  initial begin
    #5 clk = 1;
    #5 clk = 0;
    reset = 1;
    #10;
  end
  final $display("T|pending|%p", failures);
endmodule
