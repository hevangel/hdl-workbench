// top: c31_more
`timescale 1ns/1ps
module ffc (input clk, d, en, rst, output reg q);
  reg n1, n2, n3, n4, n5, n6;
  always @(posedge clk) q <= d;
  specify
    $setup(d, posedge clk &&& en, 2, n1);                // conditioned
    $hold(edge[01, 0x] clk, d, 1, n2);                    // edge descriptors
    $recrem(negedge rst, posedge clk, 3, 2, n3);
    $timeskew(posedge clk, posedge en, 2, n4);
    $fullskew(posedge clk, negedge en, 2, 2, n5);
    $width(negedge clk, 4, 1, n6);                        // threshold 1: glitches < 1 ignored
  endspecify
  always @(n1) $display("T|31.7|cond setup notifier t=%0t", $realtime);
  always @(n2) $display("T|31.5|edge hold notifier t=%0t", $realtime);
  always @(n3) $display("T|31.3.6|recrem notifier t=%0t", $realtime);
  always @(n4) $display("T|31.4.2|timeskew notifier t=%0t", $realtime);
  always @(n5) $display("T|31.4.2|fullskew notifier t=%0t", $realtime);
  always @(n6) $display("T|31.4.4|width notifier t=%0t", $realtime);
endmodule
module c31_more;
  logic clk, d, en, rst; wire q;
  ffc u (clk, d, en, rst, q);
  initial begin
    clk = 0; d = 0; en = 0; rst = 1;
    #9 d = 1; #1 clk = 1;            // t=10: setup violated but en=0 -> no report
    #5 clk = 0; #4 en = 1; #5 d = 0; #1 clk = 1; // t=25 clk with en=1, d changed at 24 -> setup
    #0.5 d = 1;                      // t=25.5 hold violation (01 edge)
    #2.5 en = 0;                     // t=28 negedge en, fullskew wrt clk 25 -> 3 > 2
    #2 clk = 0; #0.5 clk = 1; #0.2 clk = 0; // t=30 negedge, glitch high 0.5: width check
    #5 rst = 0; #1 clk = 1;          // recrem: 1ns < 3
    #10 $finish;
  end
endmodule
