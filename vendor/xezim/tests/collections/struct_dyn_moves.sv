// Moving and copying struct elements that carry a queue member.
module p8;
  typedef struct { int x; int ql[$]; } st_t;
  st_t q[$], q2[$];
  st_t aa[string], ab[string];
  st_t s;
  initial begin
    q.push_back('{x: 1, ql: '{1}});
    q.push_back('{x: 2, ql: '{2, 2}});
    q.push_front('{x: 0, ql: '{}});
    $display("T|pf %0d %0d %0d", q[0].ql.size(), q[1].ql.size(), q[2].ql.size());
    q.insert(1, '{x: 9, ql: '{9, 9, 9}});
    $display("T|ins %0d %0d %0d %0d", q[0].ql.size(), q[1].ql.size(), q[2].ql.size(), q[3].ql.size());
    q.delete(0);
    $display("T|del %0d %0d %0d n=%0d", q[0].ql.size(), q[1].ql.size(), q[2].ql.size(), q.size());
    s = q.pop_front();
    $display("T|pop x=%0d sz=%0d n=%0d", s.x, s.ql.size(), q.size());
    q2 = q;
    q2[0].ql.push_back(5);
    $display("T|qcopy %0d %0d", q2[0].ql.size(), q[0].ql.size());
    aa["a"] = '{x: 1, ql: '{1, 2}};
    ab = aa;
    ab["a"].ql.push_back(3);
    $display("T|acopy %0d %0d", ab["a"].ql.size(), aa["a"].ql.size());
    s = aa["a"];
    $display("T|elem %0d %0d", s.ql.size(), s.ql[1]);
    aa["b"] = s;
    $display("T|elemw %0d %0d", aa["b"].ql.size(), aa.num());
    $finish;
  end
endmodule
