// §18.5.8.1 size phase edges: a soft size against a hard bound, sizes
// tied to each other, a size range read from a scalar, an open size, sizes
// under implications, a scalar computed from a size, and a std::randomize
// sum of sizes.
module randomize_size_phase_edges;
  class E1; rand byte a[]; constraint c { soft a.size() == 5; a.size() < 3; } endclass
  class E3; rand byte a[]; rand byte b[]; constraint c { a.size() == b.size(); b.size() inside {[2:4]}; foreach (a[i]) a[i] == b[i] + 1; foreach (b[i]) b[i] < 100; } endclass
  class E4; rand bit [3:0] n; rand byte a[]; constraint c { a.size() inside {[n:n+3]}; n < 5; foreach (a[i]) a[i] == n; } endclass
  class E5; rand byte a[]; rand bit [3:0] k; constraint c { a.size() <= 8; foreach (a[i]) a[i] == k; } endclass
  class E6; rand bit [1:0] mode; rand int a[]; constraint c { mode == 1 -> a.size() == 4; mode == 0 -> a.size() == 0; mode inside {0, 1}; foreach (a[i]) a[i] inside {[1:3]}; } endclass
  class E7; rand int a[]; rand int s; constraint c { a.size() inside {[1:5]}; s == a.size() * 10; foreach (a[i]) a[i] == i * 2; } endclass
  initial begin
    E1 e1 = new; E3 e3 = new; E4 e4 = new; E5 e5 = new; E6 e6 = new; E7 e7 = new; int r, bad, fails, cnt;
    bad = 0; fails = 0; cnt = 0; for (int i = 0; i < 20; i++) begin r = e1.randomize(); if (!r) fails++; else begin if (e1.a.size() >= 3) bad++; end end
    $display("T|e1 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 20; i++) begin r = e3.randomize(); if (!r) fails++; else begin
      if (e3.a.size() != e3.b.size() || !(e3.b.size() inside {[2:4]})) bad++; foreach (e3.a[i]) if (e3.a[i] != e3.b[i] + 1 || e3.b[i] >= 100) bad++; end end
    $display("T|e3 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 20; i++) begin r = e4.randomize(); if (!r) fails++; else begin
      if (!(e4.a.size() inside {[e4.n:e4.n+3]}) || e4.n >= 5) bad++; foreach (e4.a[i]) if (e4.a[i] != e4.n) bad++; end end
    $display("T|e4 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 20; i++) begin r = e5.randomize(); if (!r) fails++; else begin
      if (e5.a.size() > 8) bad++; foreach (e5.a[i]) if (e5.a[i] != e5.k) bad++; end end
    $display("T|e5 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; cnt = 0; for (int i = 0; i < 20; i++) begin r = e6.randomize(); if (!r) fails++; else begin
      if (e6.mode == 1 && e6.a.size() != 4 || e6.mode == 0 && e6.a.size() != 0 || e6.mode > 1) bad++; foreach (e6.a[i]) if (!(e6.a[i] inside {[1:3]})) bad++; cnt += e6.mode; end end
    $display("T|e6 fails=%0d bad=%0d", fails, bad);
    bad = 0; fails = 0; for (int i = 0; i < 20; i++) begin r = e7.randomize(); if (!r) fails++; else begin
      if (e7.s != e7.a.size() * 10 || !(e7.a.size() inside {[1:5]})) bad++; foreach (e7.a[i]) if (e7.a[i] != i * 2) bad++; end end
    $display("T|e7 fails=%0d bad=%0d", fails, bad);
    begin int n; byte p[]; byte q[$]; bad = 0; fails = 0;
      for (int i = 0; i < 20; i++) begin
        r = std::randomize(n, p, q) with { p.size() + q.size() == n; n inside {[3:9]}; q.size() > p.size(); foreach (p[k]) p[k] == k; foreach (q[k]) q[k] == -k; };
        if (!r) fails++; else begin if (p.size() + q.size() != n || !(n inside {[3:9]}) || q.size() <= p.size()) bad++;
          foreach (p[k]) if (p[k] != k) bad++; foreach (q[k]) if (q[k] != -k) bad++; end
      end
      $display("T|s1 fails=%0d bad=%0d", fails, bad);
    end
  end
endmodule
