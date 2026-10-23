// A genvar is a constant inside its generate block (IEEE 1800-2017 27.4), so
// each instance of the assertion checks 0 < 2 or 1 < 2 and never fails.
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  int fails = 0;
  function void fail(); fails++; endfunction
  for (genvar i = 0; i < 2; i++) begin : g
    a: assert property (@(posedge clk) i < 2) else fail();
  end
  initial begin
    repeat (3) @(posedge clk);
    #1;
    if (fails == 0) $display("PASS"); else $display("FAIL: %0d assertion failures", fails);
    $finish;
  end
endmodule
