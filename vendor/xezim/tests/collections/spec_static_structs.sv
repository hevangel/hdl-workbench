class P #(type T = int, int W = 4);
  typedef struct { string s; T v; } e_t;
  typedef struct { e_t in; int k; } n_t;
  static e_t q[$];
  static e_t da[];
  static e_t aa[string];
  static e_t fa[2];
  static e_t one;
  static n_t nq[$];
  e_t mq[$];
  e_t maa[int];
  static function void put(string s, T v);
    e_t e; e.s = s; e.v = v; q.push_back(e);
  endfunction
  static function void put_aa(string k, T v);
    e_t e; e.s = k; e.v = v; aa[k] = e;
  endfunction
  static function void put_da(T v);
    e_t e; e.s = "d"; e.v = v; da = new[da.size()+1](da); da[da.size()-1] = e;
  endfunction
  static function void put_n(T v);
    n_t n; n.in.s = "n"; n.in.v = v; n.k = W; nq.push_back(n);
  endfunction
  function void mput(T v);
    e_t e; e.s = "m"; e.v = v; mq.push_back(e); maa[v] = e;
  endfunction
  static function string show();
    return $sformatf("%p", q);
  endfunction
  static function int sum();
    int t = 0; foreach (q[i]) t += q[i].v; return t;
  endfunction
endclass

class Q #(int N = 1);
  typedef struct { int a; int b; } r_t;
  static r_t q[$];
  static function void put(int a); r_t r; r.a = a; r.b = a*N; q.push_back(r); endfunction
endclass

typedef P#(int, 8) p8_t;

module tb;
  initial begin
    P#(int) o = new();
    P#(int)::put("a", 7);
    P#(int)::put("b", 9);
    $display("T|01 size=%0d s0=%s v0=%0d s1=%s v1=%0d", P#(int)::q.size(), P#(int)::q[0].s, P#(int)::q[0].v, P#(int)::q[1].s, P#(int)::q[1].v);
    $display("T|03 sum=%0d", P#(int)::sum());
    P#(int)::put_aa("x", 3);
    $display("T|04 aa num=%0d s=%s v=%0d", P#(int)::aa.num(), P#(int)::aa["x"].s, P#(int)::aa["x"].v);
    P#(int)::put_da(11);
    $display("T|05 da size=%0d v=%0d", P#(int)::da.size(), P#(int)::da[0].v);
    P#(int)::put_n(14);
    $display("T|08 nq size=%0d in.s=%s in.v=%0d k=%0d", P#(int)::nq.size(), P#(int)::nq[0].in.s, P#(int)::nq[0].in.v, P#(int)::nq[0].k);
    o.mput(15);
    $display("T|09 mq size=%0d v=%0d s=%s maa=%0d", o.mq.size(), o.mq[0].v, o.mq[0].s, o.maa[15].v);
    Q#(3)::put(5);
    $display("T|10 Q3 size=%0d a=%0d b=%0d", Q#(3)::q.size(), Q#(3)::q[0].a, Q#(3)::q[0].b);
    Q#(4)::put(6);
    $display("T|11 Q4 size=%0d a=%0d b=%0d Q3 size=%0d", Q#(4)::q.size(), Q#(4)::q[0].a, Q#(4)::q[0].b, Q#(3)::q.size());
    Q#()::put(2);
    $display("T|12 Qdef size=%0d b=%0d", Q#()::q.size(), Q#()::q[0].b);
    P#(byte)::put("y", 8'sd100);
    $display("T|14 Pbyte v=%0d s=%s", P#(byte)::q[0].v, P#(byte)::q[0].s);
    begin
      P#(int)::e_t e;
      e = P#(int)::q.pop_front();
      $display("T|15 pop s=%s v=%0d size=%0d", e.s, e.v, P#(int)::q.size());
      P#(int)::q.insert(0, e);
      $display("T|16 insert s0=%s s1=%s", P#(int)::q[0].s, P#(int)::q[1].s);
    end
    begin
      P#(int)::e_t r[$];
      r = P#(int)::q.find(x) with (x.v > 8);
      $display("T|17 find size=%0d s=%s", r.size(), r[0].s);
    end
    P#(int)::q[0].v = 99;
    $display("T|18 write v=%0d", P#(int)::q[0].v);
    P#(int)::q.delete();
    $display("T|19 deleted size=%0d", P#(int)::q.size());
    $finish;
  end
endmodule
