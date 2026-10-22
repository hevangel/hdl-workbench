// Unpacked-struct class properties assigned with patterns, with and
// without dynamic members.
module p7;
  typedef struct { int x; int y; } pl_t;
  typedef struct { int x; int ql[$]; } st_t;
  typedef struct { int z; st_t in; } nest_t;
  class C;
    pl_t a;
    st_t p;
    nest_t n;
    function new();
      a = '{x: 1, y: 2};
      p = '{x: 3, ql: '{4, 5}};
      n = '{z: 1, in: '{x: 2, ql: '{3}}};
    endfunction
    function void reset();
      p = '{x: 0, ql: '{}};
      a = '{default: 7};
    endfunction
  endclass
  C c;
  initial begin
    c = new;
    $display("T|a %0d %0d", c.a.x, c.a.y);
    $display("T|p %0d %0d %0d", c.p.x, c.p.ql.size(), c.p.ql[1]);
    $display("T|n %0d %0d %0d", c.n.z, c.n.in.x, c.n.in.ql.size());
    c.p.x = 7; c.p.ql.push_back(6);
    $display("T|p2 %0d %0d", c.p.x, c.p.ql.size());
    c.p = '{x: 9, ql: '{1}};
    $display("T|p3 %0d %0d %0d", c.p.x, c.p.ql.size(), c.p.ql[0]);
    c.a = '{x: 5, y: 6};
    $display("T|a2 %0d %0d", c.a.x, c.a.y);
    c.reset();
    $display("T|r %0d %0d %0d %0d", c.p.x, c.p.ql.size(), c.a.x, c.a.y);
    $finish;
  end
endmodule
