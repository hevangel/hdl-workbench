// n++ in an action block names the n of the module that holds the assertion
// (IEEE 1800-2017 23.9; action blocks, 16.14.1): top's n for top's assertion,
// u.n for the child's.
module child(input logic clk);
  int n = 0;
  a: assert property (@(posedge clk) 1) n++;
endmodule
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  int n = 0;
  child u(.clk(clk));
  a: assert property (@(posedge clk) 1) n++;
  initial begin
    repeat (3) @(posedge clk);
    #1;
    if (n == 3 && u.n == 3) $display("PASS");
    else $display("FAIL: top n = %0d, u.n = %0d (expect 3 and 3)", n, u.n);
    $finish;
  end
endmodule
