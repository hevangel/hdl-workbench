// top: rna2
module nansi1(a, b, y);
  input [3:0] a;
  input b;
  output [3:0] y;
  reg [3:0] y;
  always @* y = a ^ {4{b}};
endmodule
module ansi1(input [3:0] a, input b, output logic [3:0] y);
  always @* y = a ^ {4{b}};
endmodule
module rna2;
  logic [3:0] a = 4'h3; logic b = 1;
  logic [3:0] a2; logic b2;
  wire [3:0] y1, y2, y3;
  nansi1 u1(a, b, y1);
  ansi1 u2(a, b, y2);
  nansi1 u3(a2, b2, y3);
  initial begin a2 = 4'h3; b2 = 1; end
  initial #1 $display("T|r1|init-decl: nonansi=%h ansi=%h  init-proc: nonansi=%h", y1, y2, y3);
endmodule
