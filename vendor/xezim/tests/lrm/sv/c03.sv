// top: c03
// Ch3: design elements: module, program, interface, checker, package, primitive, config; time units
`timescale 1ns/1ps
package c03pkg; int pv = 7; endpackage
interface c03if; logic [3:0] sig; endinterface
primitive c03udp(output o, input a, b);
  table 0 0 : 0; 0 1 : 1; 1 0 : 1; 1 1 : 0; endtable
endprimitive
module c03sub #(parameter P=1) (input [3:0] i, output [3:0] o);
  timeunit 1ns; timeprecision 1ps;
  assign o = i + P;
endmodule
module c03;
  timeunit 1ns; timeprecision 1ps;
  c03if ifc();
  wire [3:0] o;
  wire u;
  reg ua, ub;
  c03udp g(u, ua, ub);
  c03sub #(3) s(.i(ifc.sig), .o(o));
  initial begin
    ifc.sig = 2; ua = 1; ub = 0;
    #1.5;
    $display("T|3.3|o=%0d pv=%0d", o, c03pkg::pv);
    $display("T|3.12|u=%b t=%0t rt=%0.3f", u, $time, $realtime);
    $printtimescale;
    $printtimescale(s);
  end
endmodule
