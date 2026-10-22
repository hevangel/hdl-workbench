// Structs with queue members held in queues, dynamic arrays, fixed arrays, AAs, classes.
module p2;
  typedef struct { int x; int ql[$]; } st_t;
  st_t q[$];
  st_t da[];
  st_t fa[3];
  st_t aa[int];
  st_t aas[string];
  int qc[$];
  int s;
  class C;
    st_t p;
    st_t pa[2];
    st_t paa[string];
    function new(); p = '{x: 1, ql: '{2, 3}}; endfunction
  endclass
  C c;
  initial begin
    q.push_back('{x: 1, ql: '{1, 2}});
    q.push_back('{x: 2, ql: '{3}});
    $display("T|q0 sz=%0d q1 sz=%0d e=%0d", q[0].ql.size(), q[1].ql.size(), q[1].ql[0]);
    q[0].ql.push_back(9);
    $display("T|q0push sz=%0d e2=%0d", q[0].ql.size(), q[0].ql[2]);
    q[1].ql = {4, 5, 6};
    $display("T|q1asg sz=%0d e2=%0d", q[1].ql.size(), q[1].ql[2]);
    q[1] = '{x: 3, ql: '{8}};
    $display("T|q1pat x=%0d sz=%0d", q[1].x, q[1].ql.size());
    da = new[2];
    da[1].ql.push_back(4); da[1].ql.push_back(5);
    $display("T|da1 sz=%0d e1=%0d da0=%0d", da[1].ql.size(), da[1].ql[1], da[0].ql.size());
    da[0] = '{x: 1, ql: '{7, 7, 7}};
    $display("T|da0pat sz=%0d", da[0].ql.size());
    fa[2].ql.push_back(1);
    fa[2].ql.push_back(2);
    fa[0].ql = {3};
    $display("T|fa2 sz=%0d fa0 sz=%0d fa1 sz=%0d", fa[2].ql.size(), fa[0].ql.size(), fa[1].ql.size());
    fa[1] = '{x: 0, ql: '{1, 2, 3, 4}};
    $display("T|fa1pat sz=%0d e3=%0d", fa[1].ql.size(), fa[1].ql[3]);
    aa[5].ql.push_back(10);
    aa[5].ql.push_back(11);
    aa[6] = '{x: 1, ql: '{12}};
    $display("T|aa5 sz=%0d e1=%0d aa6 sz=%0d e0=%0d n=%0d", aa[5].ql.size(), aa[5].ql[1], aa[6].ql.size(), aa[6].ql[0], aa.num());
    aas["a"].ql = {1, 2, 3};
    aas["a"].ql.push_front(0);
    $display("T|aasa sz=%0d e0=%0d e3=%0d", aas["a"].ql.size(), aas["a"].ql[0], aas["a"].ql[3]);
    aas["a"].ql.delete(0);
    void'(aas["a"].ql.pop_back());
    $display("T|aasa2 sz=%0d e0=%0d", aas["a"].ql.size(), aas["a"].ql[0]);
    qc = {aas["a"].ql, aa[5].ql, q[0].ql};
    s = 0; foreach (qc[i]) s += qc[i];
    $display("T|concat sz=%0d sum=%0d", qc.size(), s);
    aas["b"] = aas["a"];
    aas["b"].ql.push_back(50);
    $display("T|aascopy b=%0d a=%0d", aas["b"].ql.size(), aas["a"].ql.size());
    c = new;
    $display("T|cp x=%0d sz=%0d", c.p.x, c.p.ql.size());
    c.p.ql.push_back(4);
    c.pa[1].ql.push_back(1);
    c.paa["z"].ql.push_back(3); c.paa["z"].ql.push_back(4);
    $display("T|cp2 sz=%0d pa1=%0d paa=%0d e1=%0d", c.p.ql.size(), c.pa[1].ql.size(), c.paa["z"].ql.size(), c.paa["z"].ql[1]);
    c.p = '{x: 9, ql: '{}};
    $display("T|cppat x=%0d sz=%0d", c.p.x, c.p.ql.size());
    $finish;
  end
endmodule
