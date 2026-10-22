module s296;
  typedef struct packed { logic [7:0] d; logic v; } s_t;
  typedef struct packed { bit [3:0] hi; bit [11:0] lo; } w_t;
  s_t g [];
  int r;
  class C;
    rand s_t a [];
    rand w_t w [$];
    constraint c { a.size() == 3; foreach (a[i]) { a[i].d == i + 10; a[i].v == 1; }
                   w.size() == 2; foreach (w[i]) { w[i].hi == 4'h5; w[i].lo < 100; } }
    task automatic tm();
      s_t q [$];
      r = std::randomize(q) with { q.size() == 2; foreach (q[i]) { q[i].d == 8'h40 + i; q[i].v == 0; } };
      $display("T|ctask r=%0d ok=%0d", r, r && q.size() == 2 && q[1].d == 8'h41 && q[1].v == 0);
    endtask
    function automatic void fm();
      w_t a [];
      r = std::randomize(a) with { a.size() == 4; foreach (a[i]) { a[i].hi == i; a[i].lo == 12'h123 + i; } };
      $display("T|cfunc r=%0d ok=%0d", r, r && a.size() == 4 && a[3].hi == 3 && a[3].lo == 12'h126);
    endfunction
  endclass
  task automatic ta(int n);
    s_t q [$];
    w_t f [3];
    r = std::randomize(q, f) with { q.size() == n; foreach (q[i]) { q[i].d == 2 * i; q[i].v == 1; }
                                    foreach (f[i]) { f[i].hi == 4'hA; f[i].lo == q.size(); } };
    $display("T|tauto r=%0d ok=%0d", r, r && q.size() == n && q[n-1].d == 2*(n-1) && f[2].hi == 4'hA && f[2].lo == n);
  endtask
  task static tst();
    s_t a [];
    r = std::randomize(a) with { a.size() == 2; foreach (a[i]) a[i].d == 8'hF0 + i; };
    $display("T|tstatic r=%0d ok=%0d", r, r && a.size() == 2 && a[1].d == 8'hF1);
  endtask
  task automatic tnest();
    s_t a [];
    for (int k = 0; k < 3; k++) begin
      r = std::randomize(a) with { a.size() == k + 1; foreach (a[i]) { a[i].d == k; } };
      if (!(r && a.size() == k + 1 && a[k].d == k)) $display("T|tnest bad k=%0d r=%0d", k, r);
    end
    $display("T|tnest done");
  endtask
  initial begin
    C c = new;
    r = c.randomize();
    $display("T|class r=%0d ok=%0d", r, r && c.a[2].d == 12 && c.a[2].v == 1 && c.w[1].hi == 5 && c.w[1].lo < 100);
    c.tm(); c.fm();
    r = std::randomize(g) with { g.size() == 4; foreach (g[i]) { g[i].d == 3 * i; g[i].v == i % 2; } };
    $display("T|module r=%0d ok=%0d", r, r && g[3].d == 9 && g[3].v == 1);
    ta(3); ta(5); tst(); tnest();
    fork ta(2); ta(4); join
  end
endmodule
