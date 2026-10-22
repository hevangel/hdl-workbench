// IEEE 1800-2017 16.12.3: in an assertion `a ##1 b` is weak, so when the
// clock stops with `a` true at its last tick, `a ##1 b` holds and
// `not a ##1 b` fails. Expected: 1 failure (the attempt from the last tick).
module top;
  logic clk = 0, a = 0, b = 0;
  int fails = 0;
  a1: assert property (@(posedge clk) not (a ##1 b)) else fails++;
  initial begin
    a = 1;
    #5 clk = 1;   // the only tick: a = 1 is sampled, then the clock stops
    #5 clk = 0;
    #10;
  end
  final begin
    if (fails == 1) $display("PASS");
    else $display("FAIL: %0d failures, expected 1", fails);
  end
endmodule
