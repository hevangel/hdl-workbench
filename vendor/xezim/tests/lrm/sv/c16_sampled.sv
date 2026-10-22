// top: c16_sampled
module c16_sampled;
  bit clk; logic [3:0] v; bit e; logic l;
  always #5 clk = ~clk;
  initial begin
    v = 0; e = 0; l = 1'bx;
    repeat (2) @(negedge clk);
    v = 1; e = 1; l = 0; @(negedge clk);
    v = 1; e = 1; l = 1; @(negedge clk);
    v = 3; e = 0; l = 1'bx; @(negedge clk);
    v = 4'bx; @(negedge clk);
    v = 2; e = 1; l = 0; @(negedge clk);
    @(negedge clk) $finish;
  end
  always @(posedge clk)
    $display("T|16.9.3|t=%0t v=%0d past=%0d past2=%0d pastE=%0d rose=%0b fell=%0b stable=%0b changed=%0b lrose=%0b lfell=%0b lstab=%0b",
      $time, v, $past(v), $past(v,2), $past(v,1,e), $rose(e), $fell(e), $stable(v), $changed(v), $rose(l), $fell(l), $stable(l));
  a_rose: assert property (@(posedge clk) $rose(e) |-> v != 0) else $display("T|16.9.3|rose-fail t=%0t", $time);
  a_past: assert property (@(posedge clk) e |-> $past(v) <= v) else $display("T|16.9.3|past-fail t=%0t", $time);
  // global clocking
  global clocking gck @(posedge clk); endclocking
  a_gc: assert property (@($global_clock) $changed_gclk(v) |-> $past_gclk(v) !== v) else $display("T|16.9.4|gc fail t=%0t", $time);
  always @(posedge clk) $display("T|16.9.4|t=%0t rose_gclk=%0b stable_gclk=%0b future_gclk=%0d", $time, $rose_gclk(e), $stable_gclk(v), $future_gclk(v));
endmodule
