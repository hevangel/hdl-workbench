// top: c18_inh
class B; rand int x; constraint c { x inside {[1:10]}; } constraint d { x > 5; }
  function void pre_randomize(); $display("T|18.6.2|B pre"); endfunction
  function void post_randomize(); $display("T|18.6.2|B post"); endfunction endclass
class D extends B; constraint d { x < 3; }   // overrides d
  function void pre_randomize(); super.pre_randomize(); $display("T|18.6.2|D pre"); endfunction endclass
class RC; randc bit [1:0] r; endclass
class St; typedef struct { rand bit [3:0] a; rand bit [3:0] b; } s_t; rand s_t s; constraint c { s.a + s.b == 9; s.a > s.b; } endclass
class DS; rand int x; constraint s { soft x == 4; } constraint h { disable soft x; x inside {[10:12]}; } endclass
class Arr2; rand bit [3:0] m[2][3]; constraint c { foreach (m[i, j]) m[i][j] == i * 3 + j; } endclass
class Sz; rand int unsigned n; rand byte d[]; constraint c { d.size() == n; n inside {[1:3]}; foreach (d[i]) d[i] == -i; } endclass
class Null; rand B h; endclass
class Gt; rand bit [7:0] a, b; constraint c { a > b; b > 250; } endclass
class Cnd; rand bit m; rand bit [3:0] v; constraint c { m dist {0 := 1, 1 := 1}; (m == 1) -> (v == 15); if (m == 0) v < 3; else v > 13; } endclass
class SumC; rand bit [3:0] a[4]; constraint c { a.sum() with (int'(item)) == 40; foreach (a[i]) a[i] <= 10; } endclass
module c18_inh;
  B b; D d; RC rc; St st; DS ds; Arr2 a2; Sz sz; Null nl; Gt gt; Cnd cn; SumC sc; int r, bad; bit [3:0] seen;
  initial begin
    d = new; r = d.randomize(); $display("T|18.5.2|override r=%0d x<3&&x>=1=%0d", r, d.x < 3 && d.x >= 1);
    rc = new; repeat (3) begin seen = 0; repeat (4) begin void'(rc.randomize()); seen[rc.r] = 1; end $display("T|18.4.2|randc cycle full=%b", seen); end
    st = new; r = st.randomize(); $display("T|18.4|struct members r=%0d ok=%0d", r, st.s.a + st.s.b == 9 && st.s.a > st.s.b);
    ds = new; r = ds.randomize(); $display("T|18.5.13.2|disable soft r=%0d in=%0d", r, ds.x inside {[10:12]});
    a2 = new; r = a2.randomize(); $display("T|18.5.8.1|2D foreach r=%0d m=%p", r, a2.m);
    sz = new; r = sz.randomize(); $display("T|18.5.8.1|size r=%0d size==n %0d d=%p", r, sz.d.size() == sz.n, sz.d);
    nl = new; r = nl.randomize(); $display("T|18.5.8|null rand handle r=%0d", r);
    gt = new; r = gt.randomize(); $display("T|18.5|tight r=%0d a=%0d b=%0d", r, gt.a, gt.b);
    cn = new; bad = 0; repeat (100) begin void'(cn.randomize()); if (cn.m && cn.v != 15) bad++; if (!cn.m && cn.v >= 3) bad++; end $display("T|18.5.6|cond bad=%0d", bad);
    sc = new; r = sc.randomize(); $display("T|18.5.8.2|sum r=%0d sum=%0d", r, sc.a.sum() with (int'(item)));
    r = b.randomize(); $display("T|18.6|null obj randomize r=%0d", r);
  end
endmodule
