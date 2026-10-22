// §18.5.8.1: sizes tied to scalars and to each other are solved before the
// elements. Each section randomizes several times and counts failures and
// violated constraints.
module randomize_size_phase_class;
  typedef struct packed { logic v; logic [127:0] data; logic [15:0] mask; logic last; } beat_t;

  // a1: size tied to a scalar that other constraints restrict
  class A1;
    rand bit [7:0] count; rand bit [13:0] total; rand bit [2:0] sz;
    rand bit [15:0] d[];
    constraint c1 { d.size() == count + 1; total == (count + 1) << sz; sz <= 2; total <= 64; }
    constraint c2 { foreach (d[k]) d[k] == (k == count); }
    constraint o { solve count before d; }
  endclass
  // a2: two arrays, ==, <, inside
  class A2;
    rand bit [3:0] n; rand byte a[]; rand byte b[];
    constraint c { a.size() == n; b.size() < a.size(); b.size() > 0; n inside {[1:8]};
                   foreach (a[i]) a[i] == i; foreach (b[j]) b[j] == a[j] + 1; }
  endclass
  // a3: a sum of sizes
  class A3;
    rand int unsigned a[]; rand int unsigned b[]; rand bit [4:0] t;
    constraint c { a.size() + b.size() == t; t == 10; a.size() >= 2; b.size() >= 1;
                   foreach (a[i]) a[i] < 100; foreach (b[i]) b[i] > 5 && b[i] < 9; }
  endclass
  // a4: four arrays, sizes chained to two scalars and to each other
  class A4;
    rand bit [5:0] n; rand bit [5:0] m;
    rand bit [7:0] p[]; rand bit [7:0] q[]; rand bit [7:0] r[]; rand bit [7:0] s[];
    constraint c { p.size() == n; q.size() == p.size(); r.size() == m; s.size() == n + m;
                   n * 3 > 150; n < 60; m < n; m > 40;
                   foreach (p[i]) { p[i] == i; q[i] == p[i] + 1; } foreach (s[i]) s[i] < 3; }
  endclass
  // a5: struct arrays sized by a struct field, with a dist on a field
  typedef struct packed { logic [7:0] count; logic [2:0] sz; logic wr; } cmd_t;
  class A5;
    rand cmd_t cmd; rand bit [13:0] total; rand beat_t w[]; rand beat_t rd[]; rand bit dm;
    int MB = 16;
    constraint c1 { total == ((cmd.count + 1) << cmd.sz); cmd.sz <= 2; total <= 128; }
    constraint c2 { w.size() == cmd.count + 1; rd.size() == cmd.count + 1; }
    constraint c3 { dm dist {0 := 99, 1 := 1};
                    foreach (w[k]) { w[k].v == 1;
                      if (dm) w[k].mask == (1 << MB) - 1;
                      else w[k].mask dist { (1 << MB) - 1 :/ 40, [0:(1 << MB) - 2] :/ 60 }; }
                    foreach (w[k]) { if (k == cmd.count) w[k].last == 1; else w[k].last == 0; } }
    constraint o { solve cmd.count before w; }
  endclass
  // a6: nested foreach over two arrays of the same size
  class A6;
    rand bit [3:0] n; rand bit [7:0] a[]; rand bit [7:0] b[];
    constraint c { a.size() == n; b.size() == n; n inside {[2:6]};
                   foreach (a[i]) foreach (b[j]) if (i == j) a[i] == b[j] + 1; else a[i] != b[j];
                   foreach (b[j]) b[j] < 10; }
  endclass
  // a7: elements typed by a class type parameter (§8.25)
  class A7 #(type T = beat_t);
    rand bit [7:0] count; rand T w[]; rand bit [13:0] total; rand bit [2:0] sz;
    constraint c2 { w.size() == count + 1; total == (count + 1) << sz; total <= 64; sz <= 2; }
    constraint c3 { foreach (w[k]) { w[k].v == 1; w[k].last == (k == count); } }
  endclass

  function automatic bit ok1(A1 x); bit ok = x.d.size() == x.count + 1 && x.total == ((x.count+1) << x.sz) && x.sz <= 2 && x.total <= 64;
    foreach (x.d[k]) ok &= x.d[k] == (k == x.count); return ok; endfunction
  function automatic bit ok2(A2 x); bit ok = x.a.size() == x.n && x.b.size() < x.a.size() && x.b.size() > 0 && x.n inside {[1:8]};
    foreach (x.a[i]) ok &= x.a[i] == i; foreach (x.b[j]) ok &= x.b[j] == x.a[j] + 1; return ok; endfunction
  function automatic bit ok3(A3 x); bit ok = x.a.size() + x.b.size() == 10 && x.a.size() >= 2 && x.b.size() >= 1;
    foreach (x.a[i]) ok &= x.a[i] < 100; foreach (x.b[i]) ok &= x.b[i] > 5 && x.b[i] < 9; return ok; endfunction
  function automatic bit ok4(A4 x); bit ok = x.p.size() == x.n && x.q.size() == x.n && x.r.size() == x.m && x.s.size() == x.n + x.m
    && x.n * 3 > 150 && x.n < 60 && x.m < x.n && x.m > 40;
    foreach (x.p[i]) ok &= x.p[i] == i && x.q[i] == x.p[i] + 1; foreach (x.s[i]) ok &= x.s[i] < 3; return ok; endfunction
  function automatic bit ok5(A5 x); bit ok = x.total == ((x.cmd.count + 1) << x.cmd.sz) && x.cmd.sz <= 2 && x.total <= 128
    && x.w.size() == x.cmd.count + 1 && x.rd.size() == x.cmd.count + 1;
    foreach (x.w[k]) ok &= x.w[k].v == 1 && x.w[k].last == (k == x.cmd.count) && (!x.dm || x.w[k].mask == 16'hffff); return ok; endfunction
  function automatic bit ok6(A6 x); bit ok = x.a.size() == x.n && x.b.size() == x.n && x.n inside {[2:6]};
    foreach (x.a[i]) foreach (x.b[j]) ok &= (i == j) ? x.a[i] == x.b[j] + 1 : x.a[i] != x.b[j];
    foreach (x.b[j]) ok &= x.b[j] < 10; return ok; endfunction
  function automatic bit ok7(A7 x); bit ok = x.w.size() == x.count + 1 && x.total == ((x.count + 1) << x.sz) && x.total <= 64 && x.sz <= 2;
    foreach (x.w[k]) ok &= x.w[k].v == 1 && x.w[k].last == (k == x.count); return ok; endfunction

  initial begin
    A1 x1 = new; A2 x2 = new; A3 x3 = new; A4 x4 = new; A5 x5 = new; A6 x6 = new; A7 x7 = new;
    int r, bad, fails;
    bad = 0; fails = 0; for (int i = 0; i < 10; i++) begin r = x1.randomize(); if (!r) fails++; else if (!ok1(x1)) bad++; end
    $display("T|a1 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 20; i++) begin r = x2.randomize(); if (!r) fails++; else if (!ok2(x2)) bad++; end
    $display("T|a2 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 20; i++) begin r = x3.randomize(); if (!r) fails++; else if (!ok3(x3)) bad++; end
    $display("T|a3 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 10; i++) begin r = x4.randomize(); if (!r) fails++; else if (!ok4(x4)) bad++; end
    $display("T|a4 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 5; i++) begin r = x5.randomize(); if (!r) fails++; else if (!ok5(x5)) bad++; end
    $display("T|a5 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 10; i++) begin r = x6.randomize(); if (!r) fails++; else if (!ok6(x6)) bad++; end
    $display("T|a6 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 5; i++) begin r = x7.randomize(); if (!r) fails++; else if (!ok7(x7)) bad++; end
    $display("T|a7 fails=%0d bad=%0d", fails, bad);
  end
endmodule
