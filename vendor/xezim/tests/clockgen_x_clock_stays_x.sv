`timescale 1ns/1ns
// Pure-SystemVerilog reproduction of the clockgen 4-state `~` bug.
//
// An uninitialized 4-state `logic` is X at t=0. Per LRM §6.8, ~1'bx == 1'bx,
// so `always #5 clk = ~clk;` must NEVER toggle an X-start clock (no posedge,
// no edge at all). xezim's clock-generator fast path used to treat any
// non-One bit (X/Z included) as 0 and flip it to 1, synthesising a 0->1->0
// clock out of `clkX` and firing 6 posedges. The explicit `clk0 = 0` clock
// must still toggle normally (6 posedges over #60).
//
// The X clock must also stay REVIVABLE: the standard start-the-clock-after-
// reset idiom seeds it from the testbench later (`clkL` below, seeded 0 at
// t=20 -> posedges at 25,35,45,55 within #60 -> 4). And a Z-seeded clock's
// first fire is a REAL value change (~Z == X) that anyedge waiters observe,
// after which it idles at X.
//
// Run: xezim --simulate -s top tests/clockgen_x_clock_stays_x.sv
// Expect (both the reference simulator and fixed xezim): ALL four tags:
//   TAG_PASS_clock_stays_x      xezim pre-fix: TAG_FAIL_clock_active pe=6
//   TAG_PASS_explicit_toggles   both sims agree pe=6
//   TAG_PASS_late_seed          retire-based fix: TAG_FAIL_late_seed pe=0
//   TAG_PASS_z_goes_x           retire-based fix: clkZ stuck at z
module top;
  logic clkX;                     // 4-state, no init -> X at t=0
  always #5 clkX = ~clkX;         // ~X = X -> stays X, never edges

  logic clk0 = 0;                 // explicit init -> genuine clock
  always #5 clk0 = ~clk0;         // 0->1->0, posedges at 5,15,25,35,45,55

  logic clkL;                     // X until the TB seeds it at t=20
  always #5 clkL = ~clkL;
  initial begin #20; clkL = 0; end // posedges at 25,35,45,55 -> 4 by #60

  logic clkZ;                     // Z-seeded: first fire drives ~Z == X
  initial clkZ = 1'bz;
  always #5 clkZ = ~clkZ;

  int unsigned peX = 0;
  int unsigned pe0 = 0;
  int unsigned peL = 0;

  initial forever @(posedge clkX) peX++;
  initial forever @(posedge clk0) pe0++;
  initial forever @(posedge clkL) peL++;

  initial begin
    $display("CLKX t=%0t clkX=%b", $time, clkX); // report start of X clock
    #60;                    // posedges on clk0 at 5,15,25,35,45,55 -> 6
    if (peX == 0)
      $display("TAG_PASS_clock_stays_x");
    else
      $display("TAG_FAIL_clock_active peX=%0d", peX);
    if (pe0 == 6)
      $display("TAG_PASS_explicit_toggles");
    else
      $display("TAG_FAIL_explicit pe0=%0d", pe0);
    if (peL == 4)
      $display("TAG_PASS_late_seed");
    else
      $display("TAG_FAIL_late_seed peL=%0d", peL);
    if (clkZ === 1'bx)
      $display("TAG_PASS_z_goes_x");
    else
      $display("TAG_FAIL_z_goes_x clkZ=%b", clkZ);
    $finish;
  end
endmodule
