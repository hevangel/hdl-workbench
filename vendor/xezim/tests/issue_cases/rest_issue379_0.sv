// IEEE 1800-2017 12.7.1: a variable declared in a for-loop header is local to
// the loop, so after the loop, i names the module's i again.
module top;
  int i = 5;
  logic clk = 0;
  always #5 clk = ~clk;
  initial begin
    int r;
    for (int i = 0; i < 2; i++) @(posedge clk);
    r = i;   // reads top.i: 5
    i = 7;   // writes top.i
    #1;
    if (r == 5 && top.i == 7) $display("PASS");
    else $display("FAIL: i read %0d after the loop (expect 5); top.i = %0d after i = 7 (expect 7)", r, top.i);
    $finish;
  end
endmodule
