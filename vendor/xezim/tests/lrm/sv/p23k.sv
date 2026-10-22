// top: t23k
module co(output o, output [3:0] r);
  assign o = 1'b0;
  assign r = {3'b0, o};     // reads the collapsed (resolved) net
endmodule
module vo(output logic [3:0] v); initial v = 4'h5; endmodule
module t23k;
  wire w; wire [3:0] r;
  co c(.o(w), .r(r));
  assign w = 1'b1;          // output port driven from outside too
  logic [3:0] pv;
  vo u(.v(pv));             // output port into a parent variable
  initial #1 begin $display("T|23.3.3.1|w=%b r=%b", w, r); $display("T|23.3.3.2|pv=%h", pv); end
endmodule
