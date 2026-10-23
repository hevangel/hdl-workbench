// top: c16_ctl
module c16_ctl;
  bit clk, a; int fails, passes;
  always #5 clk = ~clk;
  ap: assert property (@(posedge clk) a) passes++; else fails++;
  ap2: assert property (@(posedge clk) a |-> ##3 !a) else $display("T|16|ap2 F t=%0t", $time);
  initial begin
    a = 0;
    repeat (3) @(negedge clk); $display("T|20.12|base passes=%0d fails=%0d", passes, fails);
    $assertoff(0, ap); repeat (3) @(negedge clk); $display("T|20.12|after off passes=%0d fails=%0d", passes, fails);
    $asserton(0, ap); repeat (2) @(negedge clk); $display("T|20.12|after on passes=%0d fails=%0d", passes, fails);
    $assertpassoff(0, ap); a = 1; repeat (2) @(negedge clk); $display("T|20.12|passoff passes=%0d fails=%0d", passes, fails);
    $assertpasson(0, ap); repeat (2) @(negedge clk); $display("T|20.12|passon passes=%0d fails=%0d", passes, fails);
    $assertfailoff(0, ap); a = 0; repeat (2) @(negedge clk); $display("T|20.12|failoff passes=%0d fails=%0d", passes, fails);
    $assertfailon(0, ap); repeat (2) @(negedge clk); $display("T|20.12|failon passes=%0d fails=%0d", passes, fails);
    a = 1; @(posedge clk); @(negedge clk); a = 0; #1 $assertkill(0, ap2); repeat (5) @(negedge clk);
    $display("T|20.12|after kill");
    $assertcontrol(4, 15, 7, 0, ap); repeat (2) @(negedge clk); $display("T|20.12|ctl off passes=%0d fails=%0d", passes, fails);
    $assertcontrol(3, 15, 7, 0, ap); repeat (2) @(negedge clk); $display("T|20.12|ctl on passes=%0d fails=%0d", passes, fails);
    $assertoff; repeat (2) @(negedge clk); $display("T|20.12|global off passes=%0d fails=%0d", passes, fails);
    $finish;
  end
endmodule
