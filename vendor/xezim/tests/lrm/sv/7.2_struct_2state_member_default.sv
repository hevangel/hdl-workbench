// top: rsd
module rsd;
  typedef struct { int a; logic [3:0] b = 4'h5; } s1_t;
  typedef struct { int a; logic [3:0] b; } s2_t;
  typedef struct { int a; bit [3:0] c; byte d; } s3_t;
  s1_t m1;
  s2_t m2;
  s3_t m3;
  initial begin
    $display("T|r1|a=%0d b=%h", m1.a, m1.b);
    $display("T|r2|a=%0d b=%h", m2.a, m2.b);
    $display("T|r3|a=%0d c=%h d=%0d", m3.a, m3.c, m3.d);
  end
endmodule
