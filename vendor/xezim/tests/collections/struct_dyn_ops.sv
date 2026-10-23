// %p, ==, and task/function formals of struct-with-dynamic-member types.
module p4;
  typedef struct { int x; int ql[$]; } st_t;
  typedef struct { int x; int y; } pl_t;
  st_t a, b, c;
  pl_t pa[string];
  function automatic int qsum(input st_t v);
    int s = 0;
    foreach (v.ql[i]) s += v.ql[i];
    v.ql.push_back(100); // local copy only
    return s + v.x;
  endfunction
  task automatic mk(output st_t v, input int n);
    v.x = n;
    for (int i = 0; i < n; i++) v.ql.push_back(i + 1);
  endtask
  task automatic grow(ref st_t v);
    v.ql.push_back(42);
  endtask
  function automatic st_t ret(int n);
    st_t r;
    r = '{x: n, ql: '{n, n + 1}};
    return r;
  endfunction
  initial begin
    a = '{x: 1, ql: '{2, 3}};
    $display("T|pa %p", a);
    b = a;
    $display("T|eq1 %0d", a == b);
    b.ql.push_back(4);
    $display("T|eq2 %0d", a == b);
    $display("T|sum %0d asz=%0d", qsum(a), a.ql.size());
    mk(c, 3);
    $display("T|mk x=%0d sz=%0d e2=%0d", c.x, c.ql.size(), c.ql[2]);
    grow(c);
    $display("T|grow sz=%0d e3=%0d", c.ql.size(), c.ql[3]);
    c = ret(5);
    $display("T|ret x=%0d sz=%0d e1=%0d", c.x, c.ql.size(), c.ql[1]);
    $display("T|pc %p", c);
    pa = '{"p": '{x: 1, y: 2}, "q": '{3, 4}};
    $display("T|paa %0d %0d %p", pa.num(), pa["q"].y, pa);
    $finish;
  end
endmodule
