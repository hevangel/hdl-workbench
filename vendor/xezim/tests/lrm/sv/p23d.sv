// top: t23d
// 23.3.3.3/.4/.7 net kinds across ports, interconnect
module src(output [3:0] o, input [3:0] v, input en);
  assign o = en ? v : 4'bz;
endmodule
module snk(input logic [3:0] i, output logic [3:0] o);
  assign o = i;
endmodule
module wor_out(output wor [3:0] o);
  assign o = 4'b0001;
  assign o = 4'b0100;
endmodule
module t0in(input tri0 [3:0] x, input tri1 y, output [3:0] xo, output yo);
  assign xo = x; assign yo = y;
endmodule
module pass_ic(interconnect p, output logic [3:0] o);
  snk s(.i(p), .o(o));
endmodule
module t23d;
  wand [3:0] wa;
  src s1(.o(wa), .v(4'b1100), .en(1'b1));
  src s2(.o(wa), .v(4'b1010), .en(1'b1));
  wor [3:0] wo;
  src s3(.o(wo), .v(4'b1100), .en(1'b1));
  src s4(.o(wo), .v(4'b0011), .en(1'b1));
  tri1 [3:0] t1;
  src s5(.o(t1), .v(4'b0000), .en(1'b0));
  tri0 [3:0] t0;
  src s6(.o(t0), .v(4'b0000), .en(1'b0));
  wire [3:0] wwo;
  wor_out s7(wwo);
  assign wwo = 4'b1000;
  wire [3:0] xo; wire yo;
  t0in s8(.x(), .y(), .xo(xo), .yo(yo));
  // interconnect
  interconnect ic;
  wire [3:0] ico, ico2;
  src s9(.o(ic), .v(4'h9), .en(1'b1));
  snk s10(.i(ic), .o(ico));
  interconnect ic2;
  src s11(.o(ic2), .v(4'h6), .en(1'b1));
  pass_ic s12(.p(ic2), .o(ico2));
  initial begin
    #1;
    $display("T|23.3.3.7a|wand=%b wor=%b", wa, wo);
    $display("T|23.3.3.7b|tri1=%b tri0=%b", t1, t0);
    $display("T|23.3.3.7c|wor_port=%b inner=%b", wwo, s7.o);
    $display("T|23.3.3.7d|xo=%b yo=%b", xo, yo);
    $display("T|23.3.3.4|ico=%h ico2=%h", ico, ico2);
  end
endmodule
