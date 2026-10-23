// top: t23c
// 23.3.2.x instantiation forms; implicit nets on ports; port width/sign coercion
module w8(input [7:0] i, output [7:0] o, output signed [7:0] so);
  assign o = i;
  assign so = -8'sd3;
endmodule
module sx(input signed [7:0] si, output [15:0] ext);
  assign ext = si;    // sign-extended inside: si is signed
endmodule
module drv_in(input [3:0] i, output [3:0] o);
  assign i = 4'h5;   // driving an input port from inside (coercion to inout)
  assign o = i;
endmodule
module drv_out(output [3:0] o);
endmodule
module bi(inout [3:0] io, input en, input [3:0] v);
  assign io = en ? v : 'z;
endmodule
module t23c;
  // implicit net created by port connection
  w8 u_imp(.i(8'h12), .o(imp_net), .so());
  // width mismatch: 4-bit into 8-bit input; 16-bit sink for 8-bit output
  logic [3:0] n4 = 4'hf;
  wire [15:0] wide;
  wire [3:0] narrow;
  w8 u_a(.i(n4), .o(wide), .so());
  w8 u_b(.i(16'habcd), .o(narrow), .so());
  // signed output into wider unsigned net: no sign extension across a port
  wire [15:0] sx16;
  w8 u_c(.i(8'h0), .o(), .so(sx16));
  // signed input port driven with unsigned value
  wire [15:0] ext;
  sx u_d(.si(8'h80), .ext(ext));
  // input driven from inside
  wire [3:0] dio, dio_o;
  drv_in u_e(.i(dio), .o(dio_o));
  // output port not driven
  wire [3:0] undriven;
  drv_out u_f(undriven);
  // inout with two drivers from parent and child
  wire [3:0] bus;
  logic pen = 0; logic [3:0] pv = 4'h3;
  assign bus = pen ? pv : 'z;
  logic cen = 0;
  bi u_g(bus, cen, 4'hc);
  // positional with blanks
  wire [7:0] po;
  w8 u_h(8'h44, po, );
  initial begin
    #1;
    $display("T|23.3.3a|imp=%b bits=%0d", imp_net, $bits(imp_net));
    $display("T|23.3.3b|wide=%h narrow=%h", wide, narrow);
    $display("T|23.3.3c|sx16=%h", sx16);
    $display("T|23.3.3d|ext=%h", ext);
    $display("T|23.3.3e|dio=%h dio_o=%h", dio, dio_o);
    $display("T|23.3.3f|undriven=%b", undriven);
    $display("T|23.3.3g|bus=%b", bus);
    pen = 1; #1 $display("T|23.3.3h|bus=%b", bus);
    cen = 1; #1 $display("T|23.3.3i|bus=%b", bus);
    pen = 0; #1 $display("T|23.3.3j|bus=%b io=%b", bus, u_g.io);
    $display("T|23.3.2a|po=%h", po);
  end
endmodule
