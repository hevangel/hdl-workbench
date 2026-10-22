// top: c14
`timescale 1ns/1ps
module c14;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] d = 0, q;
  logic [7:0] dout;
  wire [7:0] dbus;
  logic [7:0] drv_net;
  assign dbus = drv_net;
  int log_s[$];
  // 14.3 clocking block with skews
  clocking cb @(posedge clk);
    default input #1step output #2;
    input d;
    input #3 dl = d;        // explicit input skew + hierarchical alias expression
    output dout;
    output negedge_out = q;
  endclocking
  clocking cbn @(negedge clk);
    input #0 d0 = d;
  endclocking
  default clocking dcb @(posedge clk);
    input d;
  endclocking
  always @(posedge clk) d <= d + 1;   // d changes in NBA after the edge
  initial begin
    // 14.13 input sampling: #1step samples value before the edge
    @(cb);
    $display("T|14.13a|t=%0t cb.d=%0d d=%0d", $time, cb.d, d);
    @(cb);
    $display("T|14.13b|t=%0t cb.d=%0d d=%0d cb.dl=%0d", $time, cb.d, d, cb.dl);
    // 14.16 synchronous drive with output skew #2
    cb.dout <= 8'hab;
    $display("T|14.16a|t=%0t dout=%h", $time, dout);
    #1 $display("T|14.16b|t=%0t dout=%h", $time, dout);
    #2 $display("T|14.16c|t=%0t dout=%h", $time, dout);
    // 14.11 cycle delay ##
    ##2;
    $display("T|14.11a|t=%0t", $time);
    ##0 $display("T|14.11b|t=%0t", $time);
    // drive with cycle delay: cb.dout <= ##2 val
    cb.dout <= ##2 8'hcd;
    $display("T|14.16d|t=%0t dout=%h", $time, dout);
    @(cb); $display("T|14.16e|t=%0t dout=%h", $time, dout);
    @(cb); #3 $display("T|14.16f|t=%0t dout=%h", $time, dout);
    // #0 input skew samples in observed region
    @(cbn); $display("T|14.4|t=%0t d0=%0d d=%0d", $time, cbn.d0, d);
    // 14.10 clocking block events: @(cb.d)
    // 14.12 default clocking: ##N uses dcb
    ##1 $display("T|14.12|t=%0t dcb.d=%0d", $time, dcb.d);
    // cb event when the clocking signal edge
    begin int n = 0; fork repeat (3) @(cb) n++; join $display("T|14.13c|t=%0t n=%0d", $time, n); end
    // drive outside the clocking event -> next clock edge + skew
    #1 cb.dout <= 8'h11;
    @(posedge clk); #3 $display("T|14.16g|t=%0t dout=%h", $time, dout);
    // 14.14 global clocking
    $finish;
  end
endmodule
