// DUT for the §38.36 timed/synch callback test. `clk` has NO HDL driver — it
// is driven entirely from the VPI module, which is the cocotb arrangement.
module top;
  logic       clk;
  logic [7:0] count = 8'd0;

  always @(posedge clk) count <= count + 8'd1;

  // Deliberately no `initial` that ends the run: a pending cbAfterDelay must
  // itself keep the scheduler alive, or this finishes at time 0.
endmodule
