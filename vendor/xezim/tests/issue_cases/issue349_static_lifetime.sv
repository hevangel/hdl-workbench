// IEEE 1800-2017 §6.21, §6.8: the initializer of a `static` variable of a task
// or function runs once, before time 0, not at the first call. It reads the
// values variables and formals have then, and a call in it happens once.
// Run: xezim static_init_timing.sv
module top;
  int errors = 0;
  int g = 5;
  int calls = 0;

  function int side(); calls++; return 40; endfunction
  function int f_var(); static int n = g; n++; return n; endfunction
  function int f_formal(int a); static int n = a; n++; return n; endfunction
  function int f_call(); static int n = side(); n++; return n; endfunction

  task check(string what, int got, int want);
    if (got != want) begin
      errors++;
      $display("FAIL %s: got %0d, want %0d", what, got, want);
    end else $display("PASS %s", what);
  endtask

  initial begin
    g = 100;
    check("initializer reads g before time 0, 1st call", f_var(), 6);
    check("initializer reads g before time 0, 2nd call", f_var(), 7);
    check("initializer reads the formal's initial value, 1st call", f_formal(10), 1);
    check("initializer reads the formal's initial value, 2nd call", f_formal(20), 2);
    check("initializer with a call, 1st call", f_call(), 41);
    check("initializer with a call, 2nd call", f_call(), 42);
    check("the initializer's call runs once", calls, 1);
    if (errors == 0) $display("ALL PASS"); else $display("%0d FAILED", errors);
  end
endmodule
