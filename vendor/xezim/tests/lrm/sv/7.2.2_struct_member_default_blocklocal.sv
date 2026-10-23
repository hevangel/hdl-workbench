// top: rsd2
module rsd2;
  typedef struct { int a = 3; int b = 4; } dflt_t;
  dflt_t m1;
  initial begin
    dflt_t l1;
    $display("T|r1|%0d %0d / %0d %0d", m1.a, m1.b, l1.a, l1.b);
  end
endmodule
