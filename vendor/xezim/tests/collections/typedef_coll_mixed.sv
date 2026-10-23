package pk;
  typedef int iq_t [$];
  typedef int ia_t [string];
  typedef byte bd_t [];
  typedef struct { iq_t q; ia_t a; bd_t d; } s_t;
  typedef struct { s_t in; iq_t q2; } n_t;
endpackage
import pk::*;
class C;
  typedef int cq_t [$];
  typedef struct { cq_t q; int k; } cs_t;
  s_t s;
  cs_t cs;
  s_t sq [$];
  function void fill();
    s.q.push_back(1); s.q.push_back(2); s.a["x"] = 3; s.d = new[2];
    cs.q = {4, 5}; cs.k = 6;
    sq.push_back(s);
  endfunction
endclass
module tb;
  s_t s;
  n_t n;
  s_t arr [2];
  s_t aa [int];
  int fq [2][$];
  int fa [2][string];
  int src [$] = '{7, 8};
  initial begin
    C c = new();
    s.q = {1, 2, 3}; s.a["k"] = 4; s.d = new[3];
    $display("T|01 s q=%p a=%p dsize=%0d", s.q, s.a, s.d.size());
    n.in.q.push_back(9); n.q2 = n.in.q;
    $display("T|02 n %p %p", n.in.q, n.q2);
    arr[1].q = src; arr[1].a["z"] = 1;
    $display("T|03 arr %0d %0d %p", arr[1].q.size(), arr[1].a.num(), arr[1]);
    aa[5].q.push_back(11);
    $display("T|04 aa num=%0d q=%p", aa.num(), aa[5].q);
    fa[1]["a"] = 1; fa[1]["b"] = 2;
    fa[0] = fa[1];
    $display("T|08 fa %p num=%0d", fa[0], fa[0].num());
    fq[0] = {src, 9};
    $display("T|09 fq %p", fq[0]);
    fq[1] = fq[0].find_index with (item > 7);
    $display("T|10 fq idx %p", fq[1]);
    foreach (fq[i, j]) $display("T|11 fq[%0d][%0d]=%0d", i, j, fq[i][j]);
    s = arr[1];
    $display("T|12 copy %p", s.q);
    $display("T|13 eq %0d", s.q == arr[1].q);
    s.q.delete();
    $display("T|14 del %0d %0d", s.q.size(), arr[1].q.size());
    $display("T|15 sum %0d", arr[1].q.sum());
    $finish;
  end
endmodule
