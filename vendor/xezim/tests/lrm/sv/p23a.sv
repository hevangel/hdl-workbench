// top: t23a
// 23.2.2.1 non-ANSI port declarations
module na1(a, b, c, d);
  input [3:0] a;
  input signed [3:0] b;
  output [4:0] c;
  output d;
  reg d;
  assign c = a + b;
  always @* d = b < 0;
endmodule
module na2(.hi(x[7:4]), .lo(x[3:0]), y);
  input [7:0] x;
  output [7:0] y;
  assign y = ~x;
endmodule
module na3({p, q}, r);
  input [1:0] p;
  input [1:0] q;
  output [3:0] r;
  assign r = {q, p};
endmodule
module na4(.bus({m, n}), o);
  input [2:0] m; input [0:0] n; output [3:0] o;
  assign o = {n, m};
endmodule
module na5(a, o, q, s);
  input a;
  wire signed [7:0] a;
  output o;
  logic [5:0] o;
  output reg [3:0] q = 4'd5;
  output s;
  assign o = a;
  assign s = a < 0;
endmodule
module na6(a, b);
  input [3:0] a;
  output [3:0] b;
  wire [3:0] a;
  wire [3:0] b = a + 1;   // net declaration assignment on output
endmodule
module t23a;
  logic [3:0] a = 4'd3; logic signed [3:0] b = -4'sd2;
  wire [4:0] c; wire d;
  na1 u1(a, b, c, d);
  logic [7:0] x = 8'h3c; wire [7:0] y;
  na2 u2(.hi(x[7:4]), .lo(x[3:0]), .y(y));
  wire [7:0] y2;
  na2 u2b(x[7:4], x[3:0], y2);
  wire [3:0] r;
  na3 u3(4'b1001, r);
  wire [3:0] o4;
  na4 u4(.bus(4'b1010), .o(o4));
  logic [7:0] av = 8'hf0; wire [5:0] o5; wire [3:0] q5; wire s5;
  na5 u5(av, o5, q5, s5);
  wire [3:0] b6;
  na6 u6(4'd7, b6);
  initial begin
    #1;
    $display("T|23.2.2.1a|c=%0d d=%b", c, d);
    $display("T|23.2.2.1b|y=%h y2=%h", y, y2);
    $display("T|23.2.2.1c|r=%b", r);
    $display("T|23.2.2.1d|o4=%b", o4);
    $display("T|23.2.2.1e|o5=%b q5=%0d s5=%b bits=%0d", o5, q5, s5, $bits(u5.a));
    $display("T|23.2.2.1f|b6=%0d", b6);
    a = 4'd15; b = 4'sd7; #1 $display("T|23.2.2.1g|c=%0d d=%b", c, d);
  end
endmodule
