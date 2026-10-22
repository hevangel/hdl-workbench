typedef struct { int q[$]; int a[string]; int k; } s_t;
typedef int gq_t [$];
class C;
  s_t s;
  s_t sq [$];
  s_t sd [];
  s_t sa [int];
  s_t sf [2];
  function void fill();
    s.q.push_back(1); s.a["x"] = 3; s.k = 5;
    sq.push_back(s);
    sq.push_front(s);
    sd = new[1]; sd[0] = s;
    sa[7] = s;
    sf[1] = s;
  endfunction
  function string show(); return $sformatf("%p", s); endfunction
endclass
class D;
  typedef struct { gq_t q; int k; } gs_t;
  gs_t g;
  function void fill(); g.q.push_back(8); g.k = 2; endfunction
endclass
module tb;
  s_t s;
  int k;
  initial begin
    C c = new();
    D d = new();
    c.fill(); d.fill();
    $display("T|01 %p %p %0d", c.sq[0].q, c.sq[1].a, c.sq.size());
    $display("T|02 %p %p %p", c.sd[0].q, c.sa[7].q, c.sf[1].q);
    $display("T|03 c=%p m=%p", c.s.q, s.q);
    $display("T|04 %s", c.show());
    $display("T|05 %p", c.s);
    $display("T|06 %p %p", d.g, d.g.q);
  end
endmodule
