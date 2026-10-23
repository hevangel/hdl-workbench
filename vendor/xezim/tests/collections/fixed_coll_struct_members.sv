typedef struct packed { bit [23:0] lo; bit [23:0] hi; } rng_t;
typedef struct { int tag; rng_t sp [3:0] [$]; int iq [2] [$]; } reg_t;
typedef struct { int tag; int q [$]; } s1_t;

module tb;
  reg_t fa [2];
  reg_t s;
  rng_t mfq [3:0] [$];
  int   miq [3] [$];
  s1_t  sa [2];
  rng_t t;
  int r [$];
  rng_t rr [$];
  initial begin
    for (int i = 0; i < 8; i++) begin
      t.lo = i; t.hi = i + 3;
      if (i % 2 == 0) begin fa[1].sp[1].push_back(t); s.sp[1].push_back(t); mfq[1].push_back(t); end
      if (i % 3 == 0) begin fa[1].sp[2].push_back(t); s.sp[2].push_back(t); mfq[2].push_back(t); end
      fa[0].iq[1].push_back(i); s.iq[1].push_back(i); miq[1].push_back(i); sa[1].q.push_back(i);
    end
    $display("T|01 sizes fa=%0d %0d s=%0d %0d m=%0d %0d", fa[1].sp[1].size(), fa[1].sp[2].size(), s.sp[1].size(), s.sp[2].size(), mfq[1].size(), mfq[2].size());
    // whole assignment of a queue element of a fixed-array member
    fa[1].sp[0] = fa[1].sp[2];
    $display("T|02 fa copy size=%0d lo1=%0d", fa[1].sp[0].size(), fa[1].sp[0][1].lo);
    s.sp[0] = s.sp[2];
    $display("T|03 s copy size=%0d lo1=%0d", s.sp[0].size(), s.sp[0][1].lo);
    mfq[0] = mfq[2];
    $display("T|04 m copy size=%0d lo1=%0d", mfq[0].size(), mfq[0][1].lo);
    // find with
    fa[1].sp[3] = fa[1].sp[2].find with (item.lo > 2);
    $display("T|05 fa find size=%0d", fa[1].sp[3].size());
    s.sp[3] = s.sp[2].find with (item.lo > 2);
    $display("T|06 s find size=%0d", s.sp[3].size());
    mfq[3] = mfq[2].find with (item.lo > 2);
    $display("T|07 m find size=%0d", mfq[3].size());
    rr = fa[1].sp[2].find with (item inside {fa[1].sp[1]});
    $display("T|08 fa find-inside into var size=%0d", rr.size());
    rr = s.sp[2].find with (item inside {s.sp[1]});
    $display("T|09 s find-inside into var size=%0d", rr.size());
    rr = mfq[2].find with (item inside {mfq[1]});
    $display("T|10 m find-inside into var size=%0d", rr.size());
    $display("T|11 inside fa=%0d s=%0d m=%0d", fa[1].sp[2][2] inside {fa[1].sp[1]}, s.sp[2][2] inside {s.sp[1]}, mfq[2][2] inside {mfq[1]});
    // int queues
    r = fa[0].iq[1].find with (item > 4);
    $display("T|12 fa iq find size=%0d", r.size());
    fa[0].iq[0] = fa[0].iq[1].find with (item > 4);
    $display("T|13 fa iq find->member size=%0d", fa[0].iq[0].size());
    s.iq[0] = s.iq[1];
    $display("T|14 s iq copy size=%0d %p", s.iq[0].size(), s.iq[0]);
    miq[0] = miq[1].find_index with (item > 5);
    $display("T|15 m iq find_index %p", miq[0]);
    sa[0].q = sa[1].q.find with (item inside {[2:4]});
    $display("T|16 sa q find %p", sa[0].q);
    fa[0].iq[0] = {fa[0].iq[1], 100};
    $display("T|17 concat size=%0d last=%0d", fa[0].iq[0].size(), fa[0].iq[0][fa[0].iq[0].size()-1]);
    $display("T|18 sum=%0d min=%p", fa[0].iq[1].sum(), fa[0].iq[1].min());
    fa[0].iq[1].sort() with (-item);
    $display("T|19 sorted %p", fa[0].iq[1]);
    $display("T|20 p fa0.iq=%p", fa[0].iq);
    fa[0].iq[0].delete();
    $display("T|21 del size=%0d", fa[0].iq[0].size());
    foreach (fa[1].sp[2][k]) $display("T|22 foreach k=%0d lo=%0d", k, fa[1].sp[2][k].lo);
    foreach (s.sp[2][k]) $display("T|22b s foreach k=%0d lo=%0d", k, s.sp[2][k].lo);
    begin int n = 0; foreach (s.iq[i, j]) n++; $display("T|23b s foreach 2d n=%0d", n); end
    begin int n = 0; foreach (fa[0].iq[i, j]) n++; $display("T|23 foreach 2d n=%0d", n); end
    fa[1].sp[2] = fa[1].sp[2].find with (item inside {fa[1].sp[1]});
    $display("T|24 self find size=%0d", fa[1].sp[2].size());
    $finish;
  end
endmodule
