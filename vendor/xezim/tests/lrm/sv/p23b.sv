// top: t23b
// 23.2.2.2/23.2.2.3 ANSI ports: inheritance of direction/type/kind
typedef struct packed { logic [3:0] hi; logic [3:0] lo; } ps_t;
typedef struct { int i; byte b; } us_t;
module an1(input [3:0] a, b, input signed [3:0] sa, sb, output int sum, output logic [4:0] s,
           output reg [3:0] r = 4'd9, input var logic [3:0] v, output wire [3:0] w,
           input ps_t ps, output us_t us, input real rin, output real rout,
           input logic [3:0] arr [2], output logic [3:0] oarr [2], input string str_in, output string str_out);
  assign s = a + b;
  assign sum = sa + sb;
  assign w = v;
  assign us = '{ps.hi, ps.lo};
  assign rout = rin * 2.0;
  assign oarr[0] = arr[1];
  assign oarr[1] = arr[0];
  always_comb str_out = {str_in, "!"};
endmodule
// a port list where later ports inherit the direction but specify a new type
module an2(input logic [7:0] a, int b, output c, d, output int e, f);
  assign c = a[0];
  assign d = b[0];
  assign e = $bits(a) + $bits(b);
  assign f = $bits(c) * 10 + $bits(d);
endmodule
// ref ports
module an3(ref int cnt, ref logic [7:0] q[$]);
  initial begin #2 cnt = cnt + 100; q.push_back(8'h77); end
endmodule
// default port values
module an4(input int k = 7, input logic [3:0] m = 4'ha, output int o);
  assign o = k * 100 + m;
endmodule
module t23b;
  wire [4:0] s; int sum; logic [3:0] rr; wire [3:0] w;
  us_t us; real ro; logic [3:0] oarr [2]; string so;
  logic [3:0] arr [2] = '{4'h1, 4'h2};
  an1 u1(.a(4'd9), .b(4'd8), .sa(-4'sd3), .sb(-4'sd4), .sum(sum), .s(s), .r(rr), .v(4'h6), .w(w),
         .ps(8'hab), .us(us), .rin(1.25), .rout(ro), .arr(arr), .oarr(oarr), .str_in("hey"), .str_out(so));
  wire c, d; int e, f;
  an2 u2(8'h01, 3, c, d, e, f);
  int cnt = 5; logic [7:0] q[$];
  an3 u3(cnt, q);
  int o1, o2, o3, o4;
  an4 u4a(.o(o1));
  an4 u4b(.k(2), .o(o2));
  an4 u4c(.k(), .m(), .o(o3));
  an4 u4d(.k(3), .o(o4));
  initial begin
    #1;
    $display("T|23.2.2.2a|s=%0d sum=%0d r=%0d w=%h", s, sum, rr, w);
    $display("T|23.2.2.2b|us=%0d,%0d ro=%f oarr=%h,%h so=%s", us.i, us.b, ro, oarr[0], oarr[1], so);
    $display("T|23.2.2.3a|c=%b d=%b e=%0d f=%0d", c, d, e, f);
    $display("T|23.2.2.4|o1=%0d o2=%0d o3=%0d o4=%0d", o1, o2, o3, o4);
    #2 $display("T|23.3.3.ref|cnt=%0d q=%p", cnt, q);
  end
endmodule
