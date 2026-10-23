// top: c18_basic
class Pkt;
  rand bit [7:0] len; rand bit [3:0] kind; randc bit [2:0] rc; rand int unsigned addr; bit [7:0] nonrand;
  rand bit [7:0] payload[]; rand int arr[5];
  constraint c_len { len inside {[4:16], 32, 64}; }
  constraint c_kind { kind < 10; }
  constraint c_impl { kind == 3 -> len == 32; }
  constraint c_ifelse { if (kind > 5) addr[1:0] == 0; else addr < 1000; }
  constraint c_size { payload.size() == len % 8 + 1; }
  constraint c_fe { foreach (payload[i]) payload[i] == i + 1; foreach (arr[i]) arr[i] inside {[0:3]}; }
  constraint c_sum { arr.sum() with (int'(item)) == 7; }
  int pre_n, post_n;
  function void pre_randomize(); pre_n++; endfunction
  function void post_randomize(); post_n++; endfunction
endclass
class Dist;
  rand bit [1:0] d1; rand int d2;
  constraint cd { d1 dist {0 := 1, 1 := 1, 2 := 2, 3 :/ 0}; d2 dist {[0:9] :/ 1, [10:19] :/ 3}; }
endclass
class SB; rand bit s; rand bit [7:0] x; constraint c { solve s before x; s -> x == 0; } endclass
class Soft; rand int v; constraint s1 { soft v == 5; } constraint s2 { soft v == 7; } endclass
class Cont; rand int v; constraint c1 { v > 10; } constraint c2 { v < 5; } endclass
class Modes; rand int a, b; constraint ca { a == 1; } constraint cb { b == 2; } endclass
class StaticC; rand int v; static constraint sc { v == 3; } endclass
module c18_basic;
  Pkt p; Dist d; SB sb; Soft so; Cont co; Modes m; StaticC s1, s2;
  int ok, cnt[4], hi, lo, rcseen; bit [7:0] seen; int r, sz_ok, impl_ok, bad;
  initial begin
    p = new;
    for (int i = 0; i < 200; i++) begin
      r = p.randomize(); if (!r) bad++;
      if (!(p.len inside {[4:16],32,64}) || p.kind >= 10 || (p.kind == 3 && p.len != 32)) bad++;
      if (p.kind > 5 && p.addr[1:0] != 0) bad++; if (p.kind <= 5 && p.addr >= 1000) bad++;
      if (p.payload.size() != p.len % 8 + 1) bad++;
      foreach (p.payload[j]) if (p.payload[j] != j + 1) bad++;
      if (p.arr.sum() != 7) bad++; foreach (p.arr[j]) if (p.arr[j] > 3 || p.arr[j] < 0) bad++;
      if (i < 8) seen[p.rc] = 1;
    end
    $display("T|18.5|violations=%0d pre=%0d post=%0d randc first 8 all seen=%b", bad, p.pre_n, p.post_n, seen);
    d = new; for (int i = 0; i < 4000; i++) begin void'(d.randomize()); cnt[d.d1]++; if (d.d2 < 10) lo++; else if (d.d2 < 20) hi++; end
    $display("T|18.5.4|dist d1 ~[1000,1000,2000,0]: %0d %0d %0d %0d; d2 lo~1000 hi~3000: %0d %0d", cnt[0]/100*100, cnt[1]/100*100, cnt[2]/100*100, cnt[3], lo/250*250, hi/250*250);
    sb = new; ok = 0; for (int i = 0; i < 1000; i++) begin void'(sb.randomize()); ok += sb.s; end
    $display("T|18.5.9|solve before s~500: %0d", ok/100*100);
    so = new; r = so.randomize(); $display("T|18.5.13|soft later wins v=%0d r=%0d", so.v, r);
    r = so.randomize() with { v == 9; }; $display("T|18.5.13|soft overridden v=%0d r=%0d", so.v, r);
    co = new; co.v = 77; r = co.randomize(); $display("T|18.6|contradiction r=%0d v kept=%0d", r, co.v);
    co.c2.constraint_mode(0); r = co.randomize(); $display("T|18.9|cmode off r=%0d v>10=%0d mode=%0d", r, co.v > 10, co.c2.constraint_mode());
    m = new; m.b.rand_mode(0); m.b = 9; r = m.randomize(); $display("T|18.8|rand_mode off r=%0d a=%0d b=%0d mode=%0d", r, m.a, m.b, m.b.rand_mode());
    m.rand_mode(1); m.cb.constraint_mode(0); r = m.randomize(); $display("T|18.8|all modes on r=%0d a=%0d", r, m.a);
    s1 = new; s2 = new; s1.sc.constraint_mode(0); r = s2.randomize() with { v == 4; };
    $display("T|18.5.11|static constraint off via other inst r=%0d v=%0d", r, s2.v);
    m = new; r = m.randomize(a); $display("T|18.11|restricted list r=%0d a=%0d b=%0d", r, m.a, m.b);
    p.nonrand = 0; r = p.randomize(nonrand) with { nonrand == 200; len == 8; kind == 2; }; $display("T|18.11|nonrand made rand r=%0d nonrand=%0d", r, p.nonrand);
    r = m.randomize(null); $display("T|18.11.1|randomize(null) r=%0d", r);
    m.a = 1; m.b = 2; r = m.randomize(null); $display("T|18.11.1|randomize(null) after fix r=%0d", r);
  end
endmodule
