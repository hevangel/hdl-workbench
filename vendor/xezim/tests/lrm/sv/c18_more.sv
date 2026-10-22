// top: c18_more
class Item; rand bit [3:0] v; constraint c { v > 2; } endclass
class Holder; rand Item it; rand Item arr[3]; rand int n;
  function new(); it = new; foreach (arr[i]) arr[i] = new; endfunction
  constraint ch { it.v < 6; n == it.v * 2; foreach (arr[i]) arr[i].v == i + 3; }
endclass
class Uniq; rand bit [3:0] a[6]; constraint u { unique {a}; foreach (a[i]) a[i] < 6; } endclass
class Ordered; rand int q[5]; constraint o { foreach (q[i]) if (i > 0) q[i] > q[i-1]; foreach (q[i]) q[i] inside {[0:20]}; } endclass
class FnC; rand int x; function int lim(); return 4; endfunction constraint f { x == lim() + 1; } endclass
class LocalC; rand int x; int y = 3; endclass
class EnumC; typedef enum {A, B, C} e_t; rand e_t e; constraint ce { e != B; } endclass
class Rdq; rand bit [7:0] q[$]; constraint c { q.size() inside {[2:4]}; foreach (q[i]) q[i] == 10*i; } endclass
class StrRand; rand bit [3:0] w; string s; endclass
module c18_more;
  Holder h; Uniq u; Ordered o; FnC f; LocalC lc; EnumC ec; Rdq rq;
  int r, bad, x, y, seed_a[5], seed_b[5]; bit [7:0] b8; int cs[3];
  initial begin
    h = new; r = h.randomize(); $display("T|18.5.8|nested obj r=%0d it ok=%0d n ok=%0d arr=%0d,%0d,%0d", r, h.it.v inside {[3:5]}, h.n == h.it.v*2, h.arr[0].v, h.arr[1].v, h.arr[2].v);
    u = new; bad = 0; repeat (50) begin void'(u.randomize()); foreach (u.a[i]) foreach (u.a[j]) if (i < j && u.a[i] == u.a[j]) bad++; end
    $display("T|18.5.5|unique dup=%0d", bad);
    o = new; r = o.randomize(); bad = 0; foreach (o.q[i]) if (i > 0 && o.q[i] <= o.q[i-1]) bad++; $display("T|18.5.8.1|ordered r=%0d bad=%0d", r, bad);
    f = new; r = f.randomize(); $display("T|18.5.12|fn in constraint x=%0d", f.x);
    lc = new; y = 10; r = lc.randomize() with { x == local::y + y; }; $display("T|18.7.1|local:: x=%0d", lc.x);
    ec = new; bad = 0; repeat (100) begin void'(ec.randomize()); if (ec.e == EnumC::B) bad++; end $display("T|18.5|enum bad=%0d", bad);
    rq = new; r = rq.randomize(); $display("T|18.5.8.1|queue size ok=%0d q=%p", rq.q.size() inside {[2:4]}, rq.q);
    r = std::randomize(x, y) with { x inside {[1:5]}; y == x * 3; }; $display("T|18.12|std::randomize r=%0d ok=%0d", r, y == x*3 && x >= 1 && x <= 5);
    r = std::randomize(b8) with { b8 > 250; }; $display("T|18.12|std b8 ok=%0d", b8 > 250);
    r = std::randomize(x) with { x > 5; x < 3; }; $display("T|18.12|std contradiction r=%0d", r);
    r = randomize(x) with { x == 17; }; $display("T|18.12|scope randomize r=%0d x=%0d", r, x);
    repeat (300) randcase 1: cs[0]++; 2: cs[1]++; 0: cs[2]++; endcase
    $display("T|18.16|randcase zero weight=%0d sum=%0d ratio ok=%0d", cs[2], cs[0]+cs[1], cs[1] > cs[0]);
    begin int z = 0; randcase 0: z = 1; 0: z = 2; endcase $display("T|18.16|randcase all zero z=%0d", z); end
  end
endmodule
