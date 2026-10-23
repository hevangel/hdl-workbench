// top: c31_tc
`timescale 1ns/1ps
module ff (input clk, d, rst, en, output reg q);
  reg n_su, n_h, n_sh, n_rec, n_rem, n_w, n_p, n_sk, n_nc;
  always @(posedge clk) q <= d;
  specify
    $setup(d, posedge clk, 2, n_su);
    $hold(posedge clk, d, 1, n_h);
    $setuphold(posedge clk, d, 3, 2, n_sh);
    $recovery(negedge rst, posedge clk, 4, n_rec);
    $removal(negedge rst, posedge clk, 2, n_rem);
    $width(posedge clk, 6, 0, n_w);
    $period(posedge clk, 15, n_p);
    $skew(posedge clk, posedge en, 3, n_sk);
    $nochange(posedge clk, d, 0, 0, n_nc);
  endspecify
  always @(n_su) $display("T|31.3.1|setup notifier t=%0t %b", $realtime, n_su);
  always @(n_h) $display("T|31.3.2|hold notifier t=%0t %b", $realtime, n_h);
  always @(n_sh) $display("T|31.3.3|setuphold notifier t=%0t %b", $realtime, n_sh);
  always @(n_rec) $display("T|31.3.5|recovery notifier t=%0t %b", $realtime, n_rec);
  always @(n_rem) $display("T|31.3.4|removal notifier t=%0t %b", $realtime, n_rem);
  always @(n_w) $display("T|31.4.4|width notifier t=%0t %b", $realtime, n_w);
  always @(n_p) $display("T|31.4.3|period notifier t=%0t %b", $realtime, n_p);
  always @(n_sk) $display("T|31.4.1|skew notifier t=%0t %b", $realtime, n_sk);
  always @(n_nc) $display("T|31.4.5|nochange notifier t=%0t %b", $realtime, n_nc);
endmodule
module c31_tc;
  logic clk, d, rst, en; wire q;
  ff u (clk, d, rst, en, q);
  initial begin
    clk = 0; d = 0; rst = 1; en = 0;
    #20 clk = 1; #10 clk = 0;          // t=20 ok
    #9 d = 1; #1 clk = 1;              // t=40: d changed 1ns before: setup(2) & setuphold(3) violation
    #1 d = 0;                          // t=41: hold(1)? 1ns after = boundary; setuphold hold 2 violation
    #2 clk = 0;                        // t=43: width 3 < 6 violation
    #7 clk = 1;                        // t=50: period 10 < 15 violation
    #2 en = 1;                         // skew ok (2 < 3)
    #8 clk = 0; #5 rst = 0; #2 clk = 1; // t=65 rst neg, t=67 clk: recovery 4 violated (2 apart)
    #1 rst = 1;
    #10 clk = 0; #10 rst = 0; #1 clk = 1; // removal check
    #20 $finish;
  end
endmodule
