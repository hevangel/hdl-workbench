// §18.5.8.1, §18.12: std::randomize and class randomize over several dynamic
// arrays with foreach constraints (issue #299 neighbours), sizes tied to
// a scalar target, coupled sizes with a sum, nested foreach and queues.
module randomize_size_phase_scope;
  typedef logic [6:0] w7_t;
  typedef logic [8:0] w9_t;
  class C5;
    rand w7_t p[]; rand w7_t q[];
    constraint c { p.size() == 37; q.size() == 37; foreach (p[k]) { p[k] == k; q[k] == k; } }
  endclass
  class C6;
    rand bit [3:0] n; rand w7_t p[]; rand w7_t q[]; rand w9_t r[];
    constraint c { p.size() == n; q.size() == n; r.size() == 2 * n; n inside {[3:12]};
                   foreach (p[k]) { p[k] == k; q[k] == p[k] + 1; } foreach (r[j]) r[j] == 300 - j; }
  endclass
  task automatic t1(output int fails, output int bad);
    w7_t p[]; w7_t q[];
    fails = 0; bad = 0;
    for (int it = 0; it < 10; it++) begin
      if (std::randomize(p, q) with { p.size() == 37; q.size() == 37; foreach (p[k]) { p[k] == k; q[k] == k; } }) begin
        if (p.size() != 37 || q.size() != 37) bad++; foreach (p[k]) if (p[k] != k || q[k] != k) bad++;
      end else fails++;
    end
  endtask
  task automatic t3(output int fails, output int bad);
    int n; w7_t p[]; w7_t q[];
    fails = 0; bad = 0;
    for (int it = 0; it < 10; it++) begin
      if (std::randomize(n, p, q) with { p.size() == n; q.size() == n + 1; n inside {[3:10]};
                                          foreach (p[i]) p[i] == i; foreach (q[i]) q[i] == 2 * i; }) begin
        if (p.size() != n || q.size() != n + 1 || !(n inside {[3:10]})) bad++;
        foreach (p[i]) if (p[i] != i) bad++; foreach (q[i]) if (q[i] != 2 * i) bad++;
      end else fails++;
    end
  endtask
  task automatic t4(output int fails, output int bad);
    int p[]; int q[];
    fails = 0; bad = 0;
    for (int it = 0; it < 10; it++) begin
      if (std::randomize(p, q) with { p.size() inside {[2:5]}; q.size() == p.size() * 2;
                                       foreach (p[i]) p[i] inside {[0:9]}; foreach (q[i]) q[i] == p[i / 2]; p.sum() == 10; }) begin
        if (!(p.size() inside {[2:5]}) || q.size() != p.size() * 2 || p.sum() != 10) bad++;
        foreach (q[i]) if (q[i] != p[i / 2]) bad++; foreach (p[i]) if (!(p[i] inside {[0:9]})) bad++;
      end else fails++;
    end
  endtask
  task automatic t6(output int fails, output int bad);
    w7_t p[]; w7_t q[];
    fails = 0; bad = 0;
    for (int it = 0; it < 10; it++) begin
      if (std::randomize(p, q) with { p.size() == 6; q.size() == 6;
                                       foreach (p[i]) foreach (q[j]) if (i == j) q[j] == p[i] + 1; else q[j] != p[i];
                                       foreach (p[i]) p[i] < 20; }) begin
        if (p.size() != 6 || q.size() != 6) bad++;
        foreach (p[i]) foreach (q[j]) if ((i == j) ? q[j] != p[i] + 1 : q[j] == p[i]) bad++;
        foreach (p[i]) if (p[i] >= 20) bad++;
      end else fails++;
    end
  endtask
  task automatic t8(output int fails, output int bad);
    int qa[$]; int qb[$];
    fails = 0; bad = 0;
    for (int it = 0; it < 10; it++) begin
      if (std::randomize(qa, qb) with { qa.size() == 20; qb.size() == 20; foreach (qa[k]) { qa[k] == k; qb[k] == k * 3; } }) begin
        if (qa.size() != 20 || qb.size() != 20) bad++; foreach (qa[k]) if (qa[k] != k || qb[k] != 3 * k) bad++;
      end else fails++;
    end
  endtask
  initial begin
    int f, b; C5 c5 = new; C6 c6 = new;
    t1(f, b); $display("T|t1 task two arrays fails=%0d bad=%0d", f, b);
    // module-scope inline, four arrays (issue shape)
    begin
      w7_t s1 [], s2 [], s3 []; w9_t s4 []; int fails, bad; fails = 0; bad = 0;
      for (int it = 0; it < 10; it++) begin
        if (std::randomize(s1, s2, s3, s4) with { s1.size() == 37; s2.size() == 37; s3.size() == 37; s4.size() == 272;
              foreach (s1[k]) { s1[k] == k; s2[k] == k; s3[k] == k; } foreach (s4[k]) s4[k] == k; }) begin
          foreach (s1[k]) if (s1[k] != k || s2[k] != k || s3[k] != k) bad++; foreach (s4[k]) if (s4[k] != k) bad++;
          if (s4.size() != 272) bad++;
        end else fails++;
      end
      $display("T|t2 inline four arrays fails=%0d bad=%0d", fails, bad);
    end
    t3(f, b); $display("T|t3 sizes from scalar target fails=%0d bad=%0d", f, b);
    t4(f, b); $display("T|t4 sizes coupled + sum fails=%0d bad=%0d", f, b);
    begin int fails, bad; fails = 0; bad = 0;
      for (int it = 0; it < 10; it++) if (c5.randomize()) begin foreach (c5.p[k]) if (c5.p[k] != k || c5.q[k] != k) bad++; if (c5.p.size() != 37) bad++; end else fails++;
      $display("T|t5 class two arrays fails=%0d bad=%0d", fails, bad); end
    begin int fails, bad; fails = 0; bad = 0;
      for (int it = 0; it < 10; it++) if (c6.randomize()) begin
        if (c6.p.size() != c6.n || c6.q.size() != c6.n || c6.r.size() != 2 * c6.n || !(c6.n inside {[3:12]})) bad++;
        foreach (c6.p[k]) if (c6.p[k] != k || c6.q[k] != k + 1) bad++; foreach (c6.r[j]) if (c6.r[j] != 300 - j) bad++; end else fails++;
      $display("T|t6c class three arrays sized by n fails=%0d bad=%0d", fails, bad); end
    t6(f, b); $display("T|t6 nested foreach fails=%0d bad=%0d", f, b);
    t8(f, b); $display("T|t8 two queues fails=%0d bad=%0d", f, b);
  end
endmodule
