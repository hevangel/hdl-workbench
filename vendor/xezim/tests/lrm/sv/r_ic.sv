// top: r_ic
module src(output [3:0] o, input [3:0] v); assign o = v; endmodule
module snk(input logic [3:0] i, output logic [3:0] o); assign o = i; endmodule
module pass_ic(interconnect [3:0] p, output logic [3:0] o); snk s(.i(p), .o(o)); endmodule
module r_ic;
  interconnect [3:0] ic; wire [3:0] ico, ico2;
  src s9(.o(ic), .v(4'h9));
  snk s10(.i(ic), .o(ico));
  interconnect [3:0] ic2;
  src s11(.o(ic2), .v(4'h6));
  pass_ic s12(.p(ic2), .o(ico2));
  interconnect ic1;
  wire [3:0] ico3;
  src s13(.o(ic1), .v(4'h9));
  snk s14(.i(ic1), .o(ico3));
  initial #1 begin $display("T|ic|ico=%h ico2=%h", ico, ico2); $display("T|ic3|ico3=%b", ico3); end
endmodule
