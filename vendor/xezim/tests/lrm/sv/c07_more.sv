// top: c07x
module c07x;
  typedef struct packed { logic [3:0] op; logic [11:0] imm; } instr_t;
  typedef union packed { instr_t i; logic [15:0] raw; struct packed {logic [7:0] h; logic [7:0] l;} b; } u_t;
  u_t u;
  instr_t iarr [4];
  instr_t [1:0] ipk;           // packed array of packed struct
  typedef struct { int k; string n; } rec_t;
  rec_t recs[$];
  rec_t rmap[string];
  int qq[$][$];
  int dq[][];
  int aq[int][$];
  logic [7:0] mda [2][3][4];
  int sa[3];
  initial begin
    // 7.3.1 packed union views
    u.raw = 16'hA123; $display("T|7.3.1b|%h %h %h %h", u.i.op, u.i.imm, u.b.h, u.b.l);
    u.b.l = 8'hff; $display("T|7.3.1c|%h", u.raw);
    // arrays of packed struct
    iarr[1] = '{op: 4'h2, imm: 12'h345}; $display("T|7.4a|%h %h", iarr[1], iarr[1].imm);
    iarr[2].op = 4'hf; $display("T|7.4b|%h", iarr[2]);
    ipk = 32'h1234_5678; $display("T|7.4c|%h %h %h", ipk[1], ipk[0].op, ipk[1].imm);
    ipk[0].imm = 12'h0; $display("T|7.4d|%h", ipk);
    // queue of structs, assoc of structs
    recs.push_back('{1, "one"}); recs.push_back('{2, "two"});
    recs[0].k = 10; $display("T|7.10q|%0d %s %0d", recs[0].k, recs[1].n, recs.size());
    rmap["a"] = '{5, "five"}; rmap["a"].k++; $display("T|7.8u|%0d %s", rmap["a"].k, rmap["a"].n);
    begin rec_t found[$]; found = recs.find with (item.k > 5); $display("T|7.12.1n|%0d %0d", found.size(), found[0].k); end
    // queue of queues
    qq.push_back({1, 2}); qq.push_back({3}); qq[1].push_back(4);
    $display("T|7.10r|%p %0d", qq, qq[1].size());
    // dynamic of dynamic
    dq = new[2]; foreach (dq[i]) dq[i] = new[i + 1]; dq[1][1] = 9; $display("T|7.5g|%p", dq);
    // assoc of queues
    aq[5].push_back(1); aq[5].push_back(2); aq[7] = {3}; $display("T|7.8v|%p %0d", aq, aq[5].size());
    // multidim unpacked
    mda[1][2][3] = 8'hab; $display("T|7.4.5c|%h %0d %0d", mda[1][2][3], $size(mda, 3), $bits(mda));
    begin logic [7:0] sl [4]; sl = mda[1][2]; $display("T|7.4.5d|%h", sl[3]); end
    // slices of unpacked arrays
    sa = '{1, 2, 3};
    begin int s2[2]; s2 = sa[1:2]; $display("T|7.4.6d|%p", s2); end
    // assoc: locator methods with index
    begin int am[string] = '{"x": 5, "y": 1, "z": 5}; string ks[$]; int vs[$];
      ks = am.find_index with (item == 5); $display("T|7.12.1o|%p", ks);
      vs = am.find with (item.index == "y"); $display("T|7.12.1p|%p", vs);
      ks = am.unique_index(); $display("T|7.12.1q|%0d", ks.size());
      vs = am.min(); $display("T|7.12.1r|%p", vs); end
    // 7.9.x assoc num/exists after increment of nonexistent
    begin int cnt[string]; cnt["a"] += 1; cnt["a"] += 1; cnt["b"]++; $display("T|7.8w|%p", cnt); end
    // 7.10.4 queue update via referenced element in foreach with delete
    begin int q[$] = {1, 2, 3, 4}; foreach (q[i]) if (q[i] == 2) q.delete(i); $display("T|7.10s|%p", q); end
    // 7.10 queue slices out of range and reverse
    begin int q[$] = {1, 2, 3}; int r[$]; r = q[2:5]; $display("T|7.10t|%p", r); r = q[-1:1]; $display("T|7.10u|%p", r); end
    // 7.10 queue insert at size and beyond
    begin int q[$] = {1}; q.insert(1, 2); q.insert(5, 9); $display("T|7.10v|%p", q); end
    // 7.12.2 sort of strings, sort on queue of structs by key
    begin string ss[$] = {"pear", "apple", "fig"}; ss.sort(); $display("T|7.12.2g|%p", ss); ss.rsort(); $display("T|7.12.2h|%p", ss); end
    begin int q[$] = {3, 1, 2}; q.sort with (-item); $display("T|7.12.2i|%p", q); end
    // 7.12.3 reduction result width: logic [3:0] array sum
    begin logic [3:0] la[3] = '{4'hf, 4'hf, 4'h2}; int big; big = la.sum(); $display("T|7.12.3j|%0d", big);
      big = la.sum() with (int'(item)); $display("T|7.12.3k|%0d", big); end
    // 7.4.4 memories, x index write ignored
    begin logic [7:0] m [4]; logic [1:0] xi = 2'bx0; m = '{0, 0, 0, 0}; m[xi] = 8'hff; $display("T|7.4.6e|%p", m); end
    // 7.6 assignment between packed and unpacked: illegal; between fixed of different ranges
    begin bit [3:0] fa [1:4]; bit [3:0] fb [8:5]; fa = '{1, 2, 3, 4}; fb = fa; $display("T|7.6d|%0d %0d", fb[8], fb[5]); end
    // 7.5.2 size() on uninit, delete
    begin int d[]; $display("T|7.5.2|%0d", d.size()); end
    // 7.2.2 default member values with assignment pattern override
    begin typedef struct { int a = 3; int b = 4; } dflt_t; dflt_t d1; dflt_t d2 = '{a: 9, b: 8}; $display("T|7.2.2c|%0d %0d %0d", d1.a, d1.b, d2.a); end
    // struct compare
    begin rec_t r1 = '{1, "x"}, r2 = '{1, "x"}; $display("T|7.2.3|%0d", r1 == r2); end
    // packed struct signedness and arithmetic
    begin struct packed signed { logic [3:0] a; logic [3:0] b; } sp = 8'hff; $display("T|7.2.1k|%0d %0d", sp, sp + 1); end
  end
endmodule
