// top: c07
module c07;
  // 7.2 structs
  typedef struct { int a; logic [3:0] b = 4'h5; string s; } us_t;
  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } ps_t;
  typedef struct packed signed { logic [3:0] f1; logic [3:0] f2; } pss_t;
  typedef struct { ps_t p; int arr[2]; } nest_t;
  us_t u1, u2;
  ps_t p1;
  pss_t ps;
  nest_t n1;
  // 7.3 unions
  typedef union packed { logic [7:0] b; ps_t p; } pu_t;
  pu_t pu;
  typedef union tagged { void Invalid; int Valid; } vi_t;
  vi_t vi;
  typedef union { int i; real r; } uu_t;   // unpacked union
  uu_t uu;
  // 7.4 arrays
  logic [3:0][7:0] pa;          // packed 2D
  logic [7:0] ua [0:3];         // unpacked
  int md [2][3];
  logic [1:0][3:0] mx [2];
  bit [7:0] be [3:0];
  // 7.5 dynamic arrays
  int dyn[];
  // 7.8 associative
  int aa[string];
  int ai[int];
  byte ab[bit [3:0]];
  int awild[*];
  // 7.10 queues
  int q[$];
  int qb[$:3];
  initial begin
    // 7.2
    $display("T|7.2.1a|a=%0d b=%h s=[%s]", u1.a, u1.b, u1.s);
    u1 = '{1, 4'h2, "x"}; $display("T|7.2.1b|%p", u1);
    u2 = u1; u2.a = 9; $display("T|7.2.1c|%0d %0d", u1.a, u2.a);
    u1 = '{default: 0, s: "d"}; $display("T|7.2.2a|%p", u1);
    u1 = '{int: 7, string: "q", default: 1}; $display("T|7.2.2b|%p", u1);
    p1 = 8'hc3; $display("T|7.2.1d|%h %h %0d", p1.hi, p1.lo, $bits(p1));
    p1.lo = 4'h9; $display("T|7.2.1e|%h", p1);
    ps = 8'hf0; $display("T|7.2.1f|%0d %0d", ps, ps.f1);
    p1 = '{hi:1, lo:2}; $display("T|7.2.1g|%h", p1);
    $display("T|7.2.1h|%b", p1 == 8'h12);
    n1.p = 8'h34; n1.arr = '{5, 6}; $display("T|7.2.1i|%p", n1);
    n1 = '{p: '{hi: 4'ha, lo: 4'hb}, arr: '{default: 3}}; $display("T|7.2.1j|%p", n1);
    // 7.3
    pu.b = 8'ha5; $display("T|7.3.1|%h %h", pu.p.hi, pu.p.lo);
    vi = tagged Valid (42); $display("T|7.3.2a|%0d", vi.Valid);
    case (vi) matches
      tagged Invalid: $display("T|7.3.2b|inv");
      tagged Valid .n: $display("T|7.3.2b|valid %0d", n);
    endcase
    uu.i = 5; $display("T|7.3c|%0d", uu.i);
    // 7.4
    pa = 32'h11223344; $display("T|7.4.1a|%h %h %h", pa[0], pa[3], pa[2][3:0]);
    pa[1] = 8'hff; $display("T|7.4.1b|%h", pa);
    $display("T|7.4.1c|%h", pa[2:1]);
    ua = '{8'h1, 8'h2, 8'h3, 8'h4}; $display("T|7.4.2a|%p", ua);
    $display("T|7.4.2b|%h", ua[4]);  // out of range read -> x
    ua[5] = 8'h9;                    // ignored
    $display("T|7.4.2c|%p", ua);
    md = '{'{1,2,3}, '{4,5,6}}; $display("T|7.4.5a|%0d %0d %p", md[1][2], md[0][0], md);
    mx[1][0] = 4'hc; mx[1][1] = 4'hd; $display("T|7.4.5b|%h %h", mx[1], mx[1][1][2]);
    begin int idx = -1; $display("T|7.4.6a|%0d", md[idx][0]); end
    be = '{default: 8'hee}; $display("T|7.4.6b|%p", be);
    // 7.4.6 indexing with x
    begin logic [1:0] xi = 2'bx1; $display("T|7.4.6c|%h", ua[xi]); end
    $display("T|7.4.3|%0d %0d", $size(md), $size(md, 2));
    // 7.5 dynamic arrays
    dyn = new[3]; $display("T|7.5.1a|%p size=%0d", dyn, dyn.size());
    dyn = '{1,2,3}; dyn = new[5](dyn); $display("T|7.5.1b|%p", dyn);
    dyn = new[2](dyn); $display("T|7.5.1c|%p", dyn);
    dyn.delete(); $display("T|7.5.3|%0d", dyn.size());
    dyn = {4, 5}; $display("T|7.5d|%p", dyn);
    $display("T|7.5e|%0d", dyn[7]);
    begin int d2[][]; d2 = new[2]; d2[0] = new[3]; d2[1] = '{9}; $display("T|7.5f|%p", d2); end
    // 7.6 array assignment
    begin int a1[3] = '{1,2,3}; int a2[3]; int a3[0:2]; int a4[2:0]; a2 = a1; a1[0] = 99; a4 = a1;
      $display("T|7.6a|%p %p %0d", a2, a4, a4[2]); end
    begin int dd[]; int ff[3] = '{7,8,9}; dd = ff; $display("T|7.6b|%p", dd); q = ff; $display("T|7.6c|%p", q); end
    // 7.7 arrays as arguments - see c13
    // 7.8 associative
    aa["b"] = 2; aa["a"] = 1; aa["c"] = 3;
    $display("T|7.8a|%0d %0d %0d", aa.num(), aa.size(), aa.exists("a"));
    $display("T|7.8b|%p", aa);
    begin string k; if (aa.first(k)) $display("T|7.8c|first=%s", k);
      if (aa.next(k)) $display("T|7.8d|next=%s", k);
      if (aa.last(k)) $display("T|7.8e|last=%s", k);
      if (aa.prev(k)) $display("T|7.8f|prev=%s", k);
      k = "zz"; $display("T|7.8g|next-of-last=%0d", aa.next(k));
      k = "bb"; void'(aa.next(k)); $display("T|7.8h|next-of-nonexist=%s", k);
    end
    aa.delete("b"); $display("T|7.8i|%p", aa);
    $display("T|7.8j|nonexist=%0d", aa["nope"]);   // warning + default 0
    $display("T|7.8k|num=%0d", aa.num());
    ai[-5] = 1; ai[10] = 2; ai[3] = 3; $display("T|7.8l|%p", ai);
    begin int k2; void'(ai.first(k2)); $display("T|7.8m|%0d", k2); end
    ab[4'hf] = 1; ab[4'h1] = 2; $display("T|7.8n|%p", ab);
    begin byte k3; int r; ab[4'h3] = 5; r = ai.first(k3); $display("T|7.8o|%0d %0d", r, k3); end
    awild[5] = 1; awild[2] = 7; $display("T|7.8p|%0d", awild.num());
    begin int ac[string] = '{"x": 1, "y": 2, default: -1}; $display("T|7.8q|%0d %0d", ac["x"], ac["zz"]); end
    aa.delete(); $display("T|7.8r|%0d", aa.num());
    begin logic [7:0] al[int]; $display("T|7.8s|%h", al[3]); end
    begin int ai2[int]; ai2[1]++; ai2[1] += 5; $display("T|7.8t|%0d", ai2[1]); end
    // 7.10 queues
    q = {}; q.push_back(1); q.push_back(2); q.push_front(0); $display("T|7.10a|%p %0d", q, q.size());
    q.insert(1, 9); $display("T|7.10b|%p", q);
    $display("T|7.10c|%0d %0d", q.pop_front(), q.pop_back());
    $display("T|7.10d|%p", q);
    q.delete(0); $display("T|7.10e|%p", q);
    q = {1,2,3,4,5}; $display("T|7.10f|%p %p %p", q[1:3], q[$-1:$], q[3:1]);
    q = {q[0:1], 10, q[2:$]}; $display("T|7.10g|%p", q);
    $display("T|7.10h|%0d %0d", q[$], q[10]);
    q[q.size()] = 77; $display("T|7.10i|%p", q);
    q[10] = 3; $display("T|7.10j|%p", q);  // ignored, warning
    q.delete(); $display("T|7.10k|%0d %0d", q.size(), q.pop_front());
    qb = {1,2,3,4,5,6}; $display("T|7.10l|%p", qb);
    qb.push_back(9); $display("T|7.10m|%p", qb);
    qb.push_front(0); $display("T|7.10n|%p", qb);
    begin int qq[$] = {3,1,2}; qq.insert(3, 4); $display("T|7.10o|%p", qq); end
    begin string sq[$]; sq.push_back("a"); sq.push_back("b"); $display("T|7.10p|%p", sq); end
    // 7.12 array manipulation
    begin
      int arr[] = '{5, 3, 8, 3, 1};
      int r[$];
      int ix[$];
      r = arr.find(x) with (x > 3); $display("T|7.12.1a|%p", r);
      ix = arr.find_index with (item == 3); $display("T|7.12.1b|%p", ix);
      r = arr.find_first with (item < 5); $display("T|7.12.1c|%p", r);
      r = arr.find_last with (item < 5); $display("T|7.12.1d|%p", r);
      ix = arr.find_first_index with (item > 100); $display("T|7.12.1e|%p", ix);
      ix = arr.find_last_index(z) with (z == 3); $display("T|7.12.1f|%p", ix);
      r = arr.min(); $display("T|7.12.1g|%p", r);
      r = arr.max(); $display("T|7.12.1h|%p", r);
      r = arr.unique(); $display("T|7.12.1i|%p", r);
      ix = arr.unique_index(); $display("T|7.12.1j|%p", ix);
      r = arr.max with (item % 4); $display("T|7.12.1k|%p", r);
      r = arr.find with (item.index > 2); $display("T|7.12.1l|%p", r);
      arr.reverse(); $display("T|7.12.2a|%p", arr);
      arr.sort(); $display("T|7.12.2b|%p", arr);
      arr.rsort(); $display("T|7.12.2c|%p", arr);
      arr.sort with (item % 3); $display("T|7.12.2d|%p", arr);
      arr.shuffle(); arr.sort(); $display("T|7.12.2e|%p", arr);
      $display("T|7.12.3a|%0d %0d %0d", arr.sum(), arr.product(), arr.and());
      $display("T|7.12.3b|%0d %0d", arr.or(), arr.xor());
      $display("T|7.12.3c|%0d", arr.sum() with (item > 3));
      $display("T|7.12.3d|%0d", arr.sum() with (int'(item > 3)));
      begin byte bb[] = '{100, 100, 100}; $display("T|7.12.3e|%0d %0d", bb.sum(), bb.sum() with (int'(item))); end
      begin logic [3:0] la[3] = '{4'h1, 4'hx, 4'h2}; $display("T|7.12.3f|%h", la.sum()); end
      begin int aa2[string] = '{"a":3, "b":4}; $display("T|7.12.3g|%0d", aa2.sum()); end
      begin int e[$]; $display("T|7.12.3h|%0d %p", e.sum(), e.min()); end
      begin struct {int k; int v;} sa[3] = '{'{1,30}, '{2,10}, '{3,20}};
        sa.sort with (item.v); $display("T|7.12.2f|%0d %0d %0d", sa[0].k, sa[1].k, sa[2].k); end
      begin int m2[2][2] = '{'{1,2},'{3,4}}; $display("T|7.12.3i|%0d", m2.sum with (item.sum)); end
      begin int qa[$] = {2, 4, 6}; ix = qa.find_index with (item == item.index * 2); $display("T|7.12.4|%p", ix); end
      begin int qa2[$] = {1,2,3}; ix = qa2.find_index with (item > 1); $display("T|7.12.1m|%0d", $bits(ix[0])); end
    end
  end
endmodule
