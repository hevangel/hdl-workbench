// top: rsi
module rsi;
  typedef struct {int a; byte b;} sab_t;
  struct {int a; byte b;} m1 = '{a:5, b:6};
  sab_t m2 = '{a:5, b:6};
  sab_t m3 = '{5, 6};
  initial begin
    struct {int a; byte b;} l1 = '{a:5, b:6};
    sab_t l2 = '{a:5, b:6};
    sab_t l3 = '{5, 6};
    sab_t l4;
    struct {byte b; int a;} l5 = '{b:6, a:5};
    struct {int a; int b;} l6 = '{a:5, b:6};
    l4 = '{a:5, b:6};
    $display("T|r1|%0d %0d %0d", m1.a, m2.a, m3.a);
    $display("T|r2|%0d %0d %0d %0d", l1.a, l2.a, l3.a, l4.a);
    $display("T|r3|%0d %0d %0d", l5.a, l5.b, l6.a);
    $display("T|r4|%p", l2);
  end
endmodule
