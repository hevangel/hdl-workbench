// top: t23f
typedef struct packed { logic [3:0] a; logic [3:0] b; } ps_t;
module pm #(parameter P = 4, parameter [3:0] Q = 1, parameter signed S = 0, parameter integer I = 0,
            parameter real R = 1, parameter A = 2, parameter B = A * 3, parameter type T = int,
            parameter type T2 = T, parameter int ARR [3] = '{1, 2, 3}, parameter string STR = "def",
            parameter ps_t PS = '{4'h1, 4'h2}, parameter int MX = $, parameter UP = 5);
  localparam LB = B + 1;
  parameter BODY = 9;        // becomes local because a parameter port list exists
  T tv; T2 t2v;
  initial begin
    tv = -1; t2v = -1;
    #1;
    $display("T|%m|P=%0d bitsP=%0d Q=%0d S=%0d I=%0d R=%f", P, $bits(P), Q, S, I, R);
    $display("T|%m|A=%0d B=%0d LB=%0d BODY=%0d", A, B, LB, BODY);
    $display("T|%m|bitsT=%0d tv=%0d bitsT2=%0d t2v=%0d", $bits(T), tv, $bits(T2), t2v);
    $display("T|%m|ARR=%p STR=%s PS=%h unb=%0d", ARR, STR, PS, $isunbounded(MX));
  end
endmodule
module str_p #(parameter P = "abc"); initial #2 $display("T|%m|P=%s bits=%0d", P, $bits(P)); endmodule
module real_p #(parameter P = 1); initial #2 $display("T|%m|P=%f %0d", P, P > 1); endmodule
module sgn_p #(parameter P = 1); initial #2 $display("T|%m|P=%0d lt=%0d bits=%0d", P, P < 0, $bits(P)); endmodule
module t23f;
  function automatic int cf(int x); return x * x + 1; endfunction
  localparam int CF = cf(3);
  pm u0();
  pm #(.P(8'hff), .Q(8'hff), .S(8'hff), .I(3.7), .R(2), .A(5)) u1();
  pm #(.A(5), .B(1), .T(logic [11:0]), .ARR('{7, 8, 9}), .STR("xyz"), .PS(8'h5a), .MX(CF)) u2();
  pm #(.T(byte unsigned), .T2(shortint)) u3();
  pm #(10, 11) u4();
  str_p #("hello") s0();
  real_p #(2.5) r0();
  sgn_p #(-3) g0();
  sgn_p #(4'sb1000) g1();
  sgn_p #(3'b111) g2();
endmodule
