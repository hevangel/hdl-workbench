// top: c18_deep
class El; rand bit [3:0] v; endclass
class Cont; rand El els[]; rand int n;
  function new(); els = new[4]; foreach (els[i]) els[i] = new; endfunction
  constraint c { foreach (els[i]) els[i].v == i * 2; n == els[1].v + els[3].v; } endclass
class Bi; rand int x, y; constraint c { x == y + 1; y == 5; } endclass
class Pkt; rand bit [7:0] len; rand bit [7:0] data[]; rand bit [1:0] kind;
  constraint c1 { data.size() == len; len inside {[1:6]}; }
  constraint c2 { foreach (data[i]) if (i > 0) data[i] == data[i-1] + 1; }
  constraint c3 { kind != 3; } endclass
class Lg; rand logic [3:0] l; rand bit signed [7:0] s; constraint c { s < -100; } endclass
class Arr; rand int a[$]; constraint c { a.size() == 3; a.sum() == 30; foreach (a[i]) a[i] inside {[9:11]}; unique {a}; } endclass
module c18_deep;
  Cont ct; Bi bi; Pkt p; Lg lg; Arr ar; int r, arr[4], k; bit [3:0] b4;
  initial begin
    ct = new; r = ct.randomize(); $display("T|18.5.8|obj array r=%0d v=%0d,%0d,%0d,%0d n=%0d", r, ct.els[0].v, ct.els[1].v, ct.els[2].v, ct.els[3].v, ct.n);
    r = ct.randomize() with { els[0].v == 1; }; $display("T|18.7|with nested contradiction r=%0d", r);
    bi = new; r = bi.randomize(); $display("T|18.5|bidirectional x=%0d", bi.x);
    p = new; r = p.randomize() with { len == 4; data[0] == 250; }; $display("T|18.7|with data=%p kind<3=%0d", p.data, p.kind < 3);
    lg = new; r = lg.randomize(); $display("T|18.4|logic rand no x=%0d signed s<-100=%0d", !$isunknown(lg.l), lg.s < -100);
    ar = new; r = ar.randomize(); $display("T|18.5.5|queue unique sum r=%0d ok=%0d", r, ar.a.sum() == 30);
    r = std::randomize(arr) with { foreach (arr[i]) arr[i] == i * i; }; $display("T|18.12|std array %p", arr);
    for (k = 0; k < 3; k++) begin r = std::randomize(b4) with { b4 == k + 1; }; $display("T|18.12|loop var in with b4=%0d", b4); end
  end
endmodule
