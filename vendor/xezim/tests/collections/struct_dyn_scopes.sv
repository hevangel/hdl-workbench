// Initializers of struct / AA / struct-with-queue variables in every scope.
package pk;
  typedef struct { int x; int y; } pl_t;
  typedef struct { int x; int ql[$]; } qs_t;
  pl_t ps = '{x: 9, y: 11};
  pl_t pd = '{default: 4};
  int  pa[string] = '{"x": 11, "y": 22};
  int  pai[int] = '{1: 5, default: 3};
  pl_t pas[string] = '{"x": '{x: 1, y: 2}};
  qs_t pq = '{x: 1, ql: '{2, 3}};
  pl_t pfa[2] = '{'{1, 2}, '{3, 4}};
  pl_t pqq[$] = '{'{5, 6}};
  function automatic int get_x(); return ps.x; endfunction
endpackage

module p3t;
  import pk::*;
  pl_t ms = '{x: 1, y: 2};
  int  ma[string] = '{"a": 1};
  qs_t mq = '{x: 3, ql: '{4, 5, 6}};
  class K;
    pl_t cs = '{x: 7, y: 8};
    int  ca[string] = '{"c": 3, "d": 4};
    qs_t cq = '{x: 1, ql: '{9}};
  endclass
  K k;
  task automatic t_auto();
    pl_t ls = '{x: 21, y: 22};
    int  la[string] = '{"l": 2};
    qs_t lq = '{x: 1, ql: '{1, 2}};
    $display("T|auto ls=%0d,%0d la=%0d lq=%0d", ls.x, ls.y, la.num(), lq.ql.size());
    lq.ql.push_back(3);
  endtask
  task t_static();
    static int cnt = 0;
    static pl_t ss = '{x: 31, y: 32};
    static int sa[string] = '{"s": 1};
    static qs_t sq = '{x: 1, ql: '{1}};
    cnt++;
    $display("T|static%0d ss=%0d,%0d sa=%0d sq=%0d", cnt, ss.x, ss.y, sa.num(), sq.ql.size());
    ss.x++; sa["t"] = 2; sq.ql.push_back(2);
  endtask
  initial begin
    $display("T|pkg ps=%0d,%0d pd=%0d,%0d pa=%0d,%0d,%0d pai=%0d,%0d,%0d", ps.x, ps.y, pd.x, pd.y,
             pa.num(), pa["x"], pa["y"], pai.num(), pai[1], pai[7]);
    $display("T|pkg pas=%0d,%0d,%0d pq=%0d,%0d,%0d", pas.num(), pas["x"].x, pas["x"].y, pq.x, pq.ql.size(), pq.ql[1]);
    $display("T|pkg pfa=%0d,%0d pqq=%0d,%0d getx=%0d", pfa[0].y, pfa[1].x, pqq.size(), pqq[0].y, get_x());
    $display("T|mod ms=%0d,%0d ma=%0d,%0d mq=%0d,%0d,%0d", ms.x, ms.y, ma.num(), ma["a"], mq.x, mq.ql.size(), mq.ql[2]);
    k = new;
    $display("T|cls cs=%0d,%0d ca=%0d,%0d cq=%0d,%0d", k.cs.x, k.cs.y, k.ca.num(), k.ca["d"], k.cq.ql.size(), k.cq.ql[0]);
    t_auto(); t_auto();
    t_static();
    $finish;
  end
endmodule
