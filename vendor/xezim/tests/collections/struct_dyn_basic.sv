module p5;
  typedef struct { int x; int ql[$]; } qs_t;
  typedef struct { int x; int da[]; } ds_t;
  typedef struct { int x; int aa[string]; } as_t;
  typedef struct { int y; qs_t in; } nest_t;
  qs_t q1; ds_t d1; as_t a1; nest_t n1;
  qs_t fa[2];
  initial begin
    q1.ql.push_back(3); q1.ql = {q1.ql, 4};
    $display("T|q1 sz=%0d e1=%0d", q1.ql.size(), q1.ql[1]);
    d1.da = new[3]; d1.da[2] = 5;
    $display("T|d1 sz=%0d e2=%0d", d1.da.size(), d1.da[2]);
    a1.aa["k"] = 9; a1.aa["j"] = 8;
    $display("T|a1 n=%0d k=%0d ex=%0d", a1.aa.num(), a1.aa["k"], a1.aa.exists("j"));
    n1.in.ql.push_back(1); n1.in.ql.push_back(2);
    $display("T|n1 sz=%0d e1=%0d", n1.in.ql.size(), n1.in.ql[1]);
    fa[1].ql.push_back(7);
    $display("T|fa sz=%0d e0=%0d", fa[1].ql.size(), fa[1].ql[0]);
    $finish;
  end
endmodule
