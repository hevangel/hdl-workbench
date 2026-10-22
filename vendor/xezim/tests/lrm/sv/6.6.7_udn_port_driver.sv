// top: c06nt
package ntp;
  typedef struct { real field1; bit field2; } T;
  function automatic T Tsum(input T driver[]);
    Tsum.field1 = 0.0;
    foreach (driver[i]) Tsum.field1 += driver[i].field1;
    Tsum.field2 = driver.size() > 0;
  endfunction
  function automatic real rsum(input real d[]);
    rsum = 0.0; foreach (d[i]) rsum += d[i];
  endfunction
endpackage
module drv_r(output ntp::T o); import ntp::*; assign o = '{1.5, 1'b1}; endmodule
module c06nt;
  import ntp::*;
  nettype T wTsum with Tsum;
  nettype real wrsum with rsum;
  nettype logic [3:0] plain4;      // nettype without resolution
  wTsum ws;
  wrsum wr;
  plain4 p4;
  real a = 2.0, b = 3.25;
  assign wr = a;
  assign wr = b;
  T t1 = '{1.0, 1'b0};
  assign ws = t1;
  drv_r d1(ws);
  assign p4 = 4'h9;
  // interconnect
  interconnect ic;
  wire [3:0] icw;
  initial begin
    #1;
    $display("T|6.6.7a|wr=%f", wr);
    $display("T|6.6.7b|ws=%f %b", ws.field1, ws.field2);
    $display("T|6.6.7c|p4=%h", p4);
    a = 10.0; #1 $display("T|6.6.7d|wr=%f", wr);
  end
endmodule
