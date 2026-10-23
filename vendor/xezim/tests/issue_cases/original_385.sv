// clk starts at x and goes to 1 at time 0. A change from x to 1 is a posedge
// (IEEE 1800-2017 9.4.2, Table 9-2), so the clocking event of the cover and
// the assertion occurs at times 0, 10, 20 and 30: four attempts each.
module top;
  logic clk;
  initial begin
    clk = 1'b1;
    forever #5 clk = ~clk;
  end
  int covers = 0, passes = 0;
  function void cov(); covers++; endfunction
  function void pass(); passes++; endfunction
  c: cover property (@(posedge clk) 1) cov();
  a: assert property (@(posedge clk) 1) pass();
  initial begin
    #31;
    if (covers == 4 && passes == 4) $display("PASS");
    else $display("FAIL: cover matched %0d times, assertion passed %0d times (expect 4 and 4)", covers, passes);
    $finish;
  end
endmodule
