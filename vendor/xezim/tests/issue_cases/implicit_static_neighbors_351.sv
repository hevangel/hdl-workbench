// §6.21 / §8.6: relaxed initializers keep static storage and automatic controls.
class counts;
  function int automatic_counter(); int n = 0; n++; return n; endfunction
  static function int static_method_counter(); int n = 0; n++; return n; endfunction
endclass
module automatic auto_scope;
  function int counter(); int n = 0; n++; return n; endfunction
endmodule
module top;
  int seed = 5, calls, errors, seen, late_value;
  event ev;
  counts c = new;
  auto_scope a();
  function int mixed(); int n = 1, m = 10; n++; m += 10; return n * 100 + m; endfunction
  function automatic int automatic_counter(); int n = 0; n++; return n; endfunction
  function int supplier(); calls++; return seed; endfunction
  function int called_initializer(); int n = supplier(); n++; return n; endfunction
  function int aggregate_counter(); int n[2] = '{4, 9}; n[0]++; return n[0] + n[1]; endfunction
  always @(ev) begin : waiting
    int n = 2;
    n += 2; seen = n;
  end
  initial begin : later
    #2;
    begin int n = seed; late_value = n; end
  end
  task automatic ck(string label, int got, int want);
    if (got != want) begin errors++; $display("FAIL %s %0d %0d", label, got, want); end
    else $display("PASS %s", label);
  endtask
  initial begin
    int sum;
    seed = 100;
    #1;
    ck("mixed first", mixed(), 220); ck("mixed second", mixed(), 330);
    ck("automatic first", automatic_counter(), 1); ck("automatic second", automatic_counter(), 1);
    ck("class automatic first", c.automatic_counter(), 1); ck("class automatic second", c.automatic_counter(), 1);
    ck("static method automatic first", counts::static_method_counter(), 1);
    ck("static method automatic second", counts::static_method_counter(), 1);
    ck("automatic module first", a.counter(), 1); ck("automatic module second", a.counter(), 1);
    ck("function init before time zero", calls, 1);
    ck("called initializer first", called_initializer(), 6);
    ck("called initializer second", called_initializer(), 7);
    ck("aggregate first", aggregate_counter(), 14); ck("aggregate second", aggregate_counter(), 15);
    for (int i = 0; i < 3; i++) begin int n = i; sum += n; end
    ck("automatic loop initializer", sum, 3);
    -> ev; #1; ck("always first", seen, 4);
    -> ev; #1; ck("always second", seen, 6);
    ck("inactive block initialized before zero", late_value, 5);
    if (errors == 0) $display("ALL PASS"); else $display("FAILED %0d", errors);
    $finish;
  end
endmodule
