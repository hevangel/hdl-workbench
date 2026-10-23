// IEEE 1800-2017 §6.21: a variable declared without `static`/`automatic` in a
// static task or function, or in a block of an initial/always procedure, is
// static. Its initializer runs once, before time 0, and the value carries from
// one call (or activation) to the next.
// Run: xezim --relax-implicit-static implicit_static_init.sv
package p;
  function int pf(); int n = 0; n++; return n; endfunction
endpackage

interface ifc;
  function int f(); int n = 0; n++; return n; endfunction
endinterface

module sub(output int a, output int b);
  function int f(); int n = 0; n++; return n; endfunction
  initial begin #1; a = f(); b = f(); end
endmodule

module top;
  int errors = 0;
  int g = 5;
  int seen;
  event e;
  ifc bus();
  int sa, sb;
  sub u(sa, sb);

  function int f(); int n = 0; n++; return n; endfunction
  task static t(output int r); int n = 10; n++; r = n; endtask
  function int f_var(); int n = g; n++; return n; endfunction
  function int f_loop();
    automatic int s = 0;
    for (int i = 0; i < 3; i++) begin int n = 0; n++; s = s * 10 + n; end
    return s;
  endfunction

  always @(e) begin
    int n = 0;
    n++;
    seen = n;
  end

  task check(string what, int got, int want);
    if (got != want) begin
      errors++;
      $display("FAIL %s: got %0d, want %0d", what, got, want);
    end else $display("PASS %s", what);
  endtask

  initial begin
    int r1, r2, k2;
    for (int i = 0; i < 2; i++) begin
      int k = 0;
      k++;
      k2 = k;
    end
    g = 100;
    #2;
    check("initial loop block variable, 2nd pass", k2, 2);
    check("module function, 1st call", f(), 1);
    check("module function, 2nd call", f(), 2);
    t(r1); t(r2);
    check("task static, 1st call", r1, 11);
    check("task static, 2nd call", r2, 12);
    check("initializer reads g before time 0", f_var(), 6);
    check("loop body block variable, 1st call", f_loop(), 123);
    check("loop body block variable, 2nd call", f_loop(), 456);
    check("package function, 1st call", p::pf(), 1);
    check("package function, 2nd call", p::pf(), 2);
    check("interface function, 1st call", bus.f(), 1);
    check("interface function, 2nd call", bus.f(), 2);
    check("module instance function, 1st call", sa, 1);
    check("module instance function, 2nd call", sb, 2);
    -> e; #1 r1 = seen;
    -> e; #1 r2 = seen;
    check("always block variable, 1st activation", r1, 1);
    check("always block variable, 2nd activation", r2, 2);
    if (errors == 0) $display("ALL PASS"); else $display("%0d FAILED", errors);
  end
endmodule
