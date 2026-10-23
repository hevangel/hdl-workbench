// Dynamic / associative / queue members, nested structs, pattern forms.
module p1;
  typedef struct { int x; int ql[$]; } qs_t;
  typedef struct { int x; int da[]; } ds_t;
  typedef struct { int x; int aa[string]; } as_t;
  typedef struct { int y; qs_t in; } nest_t;
  typedef struct { int x; int ql[$]; int da[]; } mix_t;

  qs_t q1 = '{x: 3, ql: '{4, 5, 6}};
  ds_t d1 = '{x: 1, da: '{7, 8}};
  as_t a1 = '{x: 2, aa: '{"k": 9}};
  nest_t n1 = '{y: 1, in: '{x: 2, ql: '{3, 4}}};
  qs_t qd;
  ds_t dd;
  as_t ad;
  nest_t n2;
  mix_t m1;
  int s;

  initial begin
    $display("T|q1init x=%0d sz=%0d e2=%0d", q1.x, q1.ql.size(), q1.ql[2]);
    $display("T|d1init x=%0d sz=%0d e1=%0d", d1.x, d1.da.size(), d1.da[1]);
    $display("T|a1init x=%0d n=%0d k=%0d", a1.x, a1.aa.num(), a1.aa["k"]);
    $display("T|n1init y=%0d x=%0d sz=%0d e1=%0d", n1.y, n1.in.x, n1.in.ql.size(), n1.in.ql[1]);
    // procedural pattern over a populated value
    d1.da = new[5];
    d1 = '{x: 4, da: '{1, 2, 3}};
    $display("T|d1pat x=%0d sz=%0d e2=%0d", d1.x, d1.da.size(), d1.da[2]);
    a1.aa["z"] = 1;
    a1 = '{x: 5, aa: '{"m": 3, "n": 4}};
    $display("T|a1pat x=%0d n=%0d ex_z=%0d m=%0d", a1.x, a1.aa.num(), a1.aa.exists("z"), a1.aa["m"]);
    n1.in.ql.push_back(99);
    n1 = '{y: 7, in: '{x: 8, ql: '{1}}};
    $display("T|n1pat y=%0d x=%0d sz=%0d e0=%0d", n1.y, n1.in.x, n1.in.ql.size(), n1.in.ql[0]);
    // nested member queue ops
    n2.in.ql.push_back(5); n2.in.ql.push_back(6);
    $display("T|n2push sz=%0d e1=%0d", n2.in.ql.size(), n2.in.ql[1]);
    n2.in = '{x: 1, ql: '{2, 3, 4}};
    $display("T|n2inpat x=%0d sz=%0d e2=%0d", n2.in.x, n2.in.ql.size(), n2.in.ql[2]);
    n2.in.ql = {};
    $display("T|n2clr sz=%0d", n2.in.ql.size());
    // default patterns: dynamic members become empty
    qd.ql = {1, 2}; qd.x = 9;
    qd = '{default: 0, ql: '{}};
    $display("T|qddef x=%0d sz=%0d", qd.x, qd.ql.size());
    m1.ql = {1}; m1.da = new[3];
    m1 = '{x: 2, ql: '{}, da: '{0}};
    $display("T|m1def x=%0d qsz=%0d dsz=%0d", m1.x, m1.ql.size(), m1.da.size());
    // pattern with an expression operand of queue type
    qd.ql = {5, 6, 7};
    q1 = '{x: 1, ql: qd.ql};
    qd.ql.push_back(8);
    $display("T|q1fromq sz=%0d e2=%0d qdsz=%0d", q1.ql.size(), q1.ql[2], qd.ql.size());
    // copy of nested struct carries the queue by value
    n2 = n1;
    n2.in.ql.push_back(10);
    $display("T|n2copy sz=%0d n1sz=%0d", n2.in.ql.size(), n1.in.ql.size());
    // dynamic-array member copy
    dd = d1;
    dd.da[0] = 77;
    $display("T|ddcopy sz=%0d e0=%0d d1e0=%0d", dd.da.size(), dd.da[0], d1.da[0]);
    ad = a1;
    ad.aa["q"] = 1;
    $display("T|adcopy n=%0d a1n=%0d", ad.aa.num(), a1.aa.num());
    s = 0; foreach (n1.in.ql[i]) s += n1.in.ql[i];
    $display("T|n1sum %0d", s);
    $finish;
  end
endmodule
