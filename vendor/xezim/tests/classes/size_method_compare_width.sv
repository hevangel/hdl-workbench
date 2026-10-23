// §7.5.1, §7.10.2, §11.6.1: `size()` and `num()` return an int, so a
// comparison with a narrower expression is evaluated at 32 bits:
// `s.size() == n + m` over 6-bit n, m adds without wrapping.
module size_method_compare_width;
  class A; bit [5:0] n; bit [5:0] m; byte s[]; int aa[int];
    function void f(); $display("T|method %0d %0d", s.size() == n + m, aa.num() == n - m + 6); endfunction
  endclass
  function automatic bit chk(A x); return x.s.size() == x.n + x.m; endfunction
  task automatic t(A x); $display("T|task %0d", x.s.size() == x.n + x.m); endtask
  bit [5:0] n2, m2; byte s2[]; int q[$];
  initial begin A x = new; bit ok; x.n = 57; x.m = 52; x.s = new[109]; x.aa[1] = 1;
    n2 = 57; m2 = 52; s2 = new[109]; q = '{1, 2, 3};
    x.f();
    $display("T|module %0d %0d %0d", s2.size() == n2 + m2, (n2 + m2) == s2.size(), q.size() == n2 + m2 - 106);
    $display("T|handle %0d %0d", x.s.size() == x.n + x.m, x.s.size() == 109);
    ok = x.s.size() == x.n + x.m;
    $display("T|assign %0d func %0d", ok, chk(x));
    t(x);
    $display("T|narrow %0d", 6'(x.s.size()) == x.n + x.m);
  end
endmodule
