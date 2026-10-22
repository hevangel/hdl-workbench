// top: cleft
`timescale 1ns/1ns
module cleft;
  logic clk = 0;
  always #5 clk = ~clk;
  global clocking gclk @(posedge clk); endclocking
  int gcnt = 0;
  always @(gclk) gcnt++;
  typedef enum {A, B} e_t;
  int fails = 0;
  initial begin
    e_t e;
    // 20.11 assertion control on immediate assertions
    $assertoff;
    assert (0) else fails++;
    $asserton;
    assert (0) else fails++;
    $display("T|20.11|fails=%0d", fails);
    // 6.24.2 $cast as a task with an invalid value: runtime error, e unchanged
    e = B;
    $cast(e, 5);
    $display("T|6.24.2d|e=%s", e.name());
    #31 $display("T|14.14|gcnt=%0d", gcnt);
    $finish;
  end
endmodule
