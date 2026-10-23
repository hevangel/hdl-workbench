// top: c18_ext2
class C; rand int x, y;
  constraint cx;            // implicit external
  extern constraint cy;     // explicit external
endclass
constraint C::cx { x inside {[1:3]}; }
constraint C::cy { y == x * 2; }
class Dd; rand bit [3:0] v; constraint c { v dist { 0 := 5, [1:2] :/ 5 }; } endclass
class Hnd; rand int a; int ref_val = 4; endclass
module c18_ext2;
  C c; Dd d; Hnd h; int r, cnt[16], other; int ref_val = 9;
  initial begin
    c = new; r = c.randomize(); $display("T|18.5.1|external r=%0d x in=%0d y==2x=%0d", r, c.x inside {[1:3]}, c.y == 2 * c.x);
    d = new; repeat (2000) begin void'(d.randomize()); cnt[d.v]++; end
    other = 0; for (int i = 3; i < 16; i++) other += cnt[i];
    $display("T|18.5.4|dist default: zero~%0d onetwo~%0d other_nonzero=%0d", cnt[0] > 700, (cnt[1] + cnt[2]) > 700, other > 50);
    h = new; r = h.randomize() with { a == ref_val; }; $display("T|18.7|with name resolution object first a=%0d (expect 4)", h.a);
    r = h.randomize() with { a == local::ref_val; }; $display("T|18.7.1|local:: a=%0d (expect 9)", h.a);
    h.a = 5; h.a.rand_mode(0); r = h.randomize() with { a == 6; }; $display("T|18.8|with on non-rand r=%0d a=%0d", r, h.a);
  end
endmodule
