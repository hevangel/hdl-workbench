// Casts to a typedef (IEEE 1800-2017 6.18, 6.24.1).
module sub(output int w, output logic neg);
  typedef logic signed [15:0] t;     // same name as top's t
  logic [15:0] x = 16'hFFFE;
  assign w   = $bits(t'(x));         // 16: sub's t
  assign neg = t'(x) < 0;            // 1:  sub's t is signed
endmodule
module top;
  typedef logic [3:0] t;
  typedef logic signed [7:0] sint_t;
  typedef sint_t sint2_t;            // a typedef of a signed typedef
  logic [7:0] x = 8'hFE;
  logic clk = 0;
  logic c1, c2, f1, f2, i1, i2, neg;
  int w;
  sub u(.w(w), .neg(neg));
  always_comb c1 = sint_t'(x) < 0;
  always_comb c2 = sint2_t'(x) < 0;
  always @(posedge clk) begin
    f1 <= sint_t'(x) < 0;
    f2 <= sint2_t'(x) < 0;
  end
  initial begin
    #1 clk = 1;
    #1;
    i1 = sint_t'(x) < 0;
    i2 = sint2_t'(x) < 0;
    $display("check 1, sint_t'(x) < 0 / sint2_t'(x) < 0: always_comb %b/%b, always @(posedge) %b/%b, initial %b/%b",
             c1, c2, f1, f2, i1, i2);
    $display("check 2, in sub: $bits(t'(x)) = %0d, t'(x) < 0 = %b", w, neg);
    if ({c1, c2, f1, f2, i1, i2} === 6'b111111 && w == 16 && neg === 1'b1) $display("PASS");
    else $display("FAIL");
  end
endmodule
