// top: c23
// 23.2 non-ANSI ports
module nansi(a, b, y, .alias_p(inner));
  input [3:0] a;
  input b;
  output [3:0] y;
  output inner;
  reg [3:0] y;
  wire inner = b;
  always @* y = a ^ {4{b}};
endmodule
// ANSI with defaults, var output, implicit type inheritance
module ansi #(parameter W = 4, parameter type T = logic [7:0], parameter int P2 = W * 2)
  (input [W-1:0] a, b, output logic [W-1:0] y, output T tv, input int unused = 5, output int pout);
  assign y = a + b;
  assign tv = T'(P2);
  assign pout = unused;
endmodule
// port coercion: output port driven from outside (inout-like), unconnected inputs
module coer(input [3:0] i, output [3:0] o, input [3:0] ub);
  assign o = i;
  initial #1 $display("T|23.3.3|ub=%b", ub);
endmodule
// size mismatch at port
module szm(input [7:0] i8, output [7:0] o8);
  assign o8 = i8;
endmodule
// upward name resolution
module leaf;
  int lv = 11;
  initial #2 $display("T|23.8a|up=%0d", mid.mv);
  task show; $display("T|23.8b|leaf task called"); endtask
endmodule
module mid;
  int mv = 22;
  leaf l1();
endmodule
// nested module
module outer;
  module inner_m; int nv = 33; endmodule
  inner_m im();
  initial #2 $display("T|23.4|nested=%0d", im.nv);
endmodule
// defparam target
module dp #(parameter X = 1, parameter Y = 2);
  initial #1 $display("T|23.10.1|%m X=%0d Y=%0d", X, Y);
endmodule
// bind target
module bindme(input [3:0] sig);
  initial #3 $display("T|23.11|bound sees %0d in %m", sig);
endmodule
module bound_tgt; logic [3:0] s = 9; endmodule
module c23;
  logic [3:0] a = 4'h3, nb = 4'h5;
  logic b = 1;
  wire [3:0] y1, y2;
  wire in1;
  wire [7:0] tvo;
  int po;
  nansi u_n(a, b, y1, in1);
  ansi #(.W(4)) u_a(.a(a), .b(nb), .y(y2), .tv(tvo), .pout(po));
  // .* and .name
  logic [3:0] i = 4'h7;
  wire [3:0] o;
  wire [3:0] ub;
  coer u_c(.i, .o, .ub());
  wire [3:0] o2;
  coer u_c2(.*, .o(o2), .ub(4'h1));
  // size mismatch
  wire [3:0] small4 = 4'hf;
  wire [11:0] big12;
  szm u_s(.i8(small4), .o8(big12));
  mid u_mid();
  outer u_out();
  dp d0();
  dp #(5) d1();
  dp #(.Y(7)) d2();
  dp #(3, 4) d3();
  dp d4();
  defparam d4.X = 100, d4.Y = 200;
  bound_tgt bt();
  bind bound_tgt bindme bm(.sig(s));
  // 23.6 hierarchical names into generate and instance arrays
  ansi #(.W(2)) arr_i [1:0] (.a(2'b01), .b(2'b01), .y(), .tv(), .pout());
  initial begin
    #1;
    $display("T|23.2.2a|y1=%h in1=%b", y1, in1);
    $display("T|23.2.2b|y2=%h tv=%0d po=%0d", y2, tvo, po);
    $display("T|23.3.2|o=%h o2=%h", o, o2);
    $display("T|23.3.3b|big12=%h", big12);
    $display("T|23.6a|%0d %0d", u_mid.l1.lv, $root.c23.u_mid.mv);
    u_mid.l1.show();
    u_mid.l1.lv = 99; #0 $display("T|23.6b|%0d", c23.u_mid.l1.lv);
    $display("T|23.6c|%0d", arr_i[1].y);
    $display("T|23.6d|%0d", $bits(u_a.a));
  end
endmodule
