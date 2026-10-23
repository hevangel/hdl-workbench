// top: rna
module nansi1(a, b, y);
  input [3:0] a;
  input b;
  output [3:0] y;
  reg [3:0] y;
  always @* y = a ^ {4{b}};
endmodule
module nansi2(a, y, .alias_p(inner));
  input [3:0] a;
  output [3:0] y;
  output inner;
  wire inner = a[0];
  assign y = ~a;
endmodule
module nansi3(a, b, y);
  input [3:0] a;
  input b;
  output reg [3:0] y;
  always @* y = a ^ {4{b}};
endmodule
module rna;
  logic [3:0] a = 4'h3; logic b = 1;
  wire [3:0] y1, y2, y3; wire i2;
  nansi1 u1(a, b, y1);
  nansi2 u2(a, y2, i2);
  nansi3 u3(a, b, y3);
  initial #1 $display("T|r1|y1=%h y2=%h i2=%b y3=%h", y1, y2, i2, y3);
endmodule
