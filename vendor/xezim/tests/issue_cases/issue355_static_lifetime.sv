// IEEE 1800-2017 §6.21, §13.3, §13.4: a variable declared in a static task or
// function, or declared `static`, is allocated once and shared by all calls.
// That holds for unpacked arrays, queues and associative arrays too.
// Run: xezim static_array_locals.sv
module top;
  int errors = 0;

  task check(string what, int got, int want);
    if (got != want) begin
      errors++;
      $display("FAIL %s: got %0d, want %0d", what, got, want);
    end else $display("PASS %s", what);
  endtask

  // A static function's fixed array, 2-D array, queue and associative array.
  function int count(bit clear);
    int m [2];
    if (clear) m[0] = 0;
    m[0] = m[0] + 1;
    return m[0];
  endfunction
  function int grid(bit wr, int v);
    int g [2][3];
    if (wr) g[1][2] = v;
    return g[1][2];
  endfunction
  function int queue_len(int v);
    int q [$];
    q.push_back(v);
    return q.size();
  endfunction
  function int assoc_num(string k);
    int a [string];
    a[k] = 1;
    return a.num();
  endfunction

  // A static task's array.
  task sum_task(int v, output int s);
    int t [4];
    t[v] = v + 1;
    s = t[0] + t[1] + t[2] + t[3];
  endtask

  // A `static` array of an automatic function; its initializer runs once.
  function automatic int explicit_static();
    static int m [2] = '{10, 20};
    m[0]++;
    return m[0];
  endfunction

  // Recursive calls of a static function share its array.
  function int rec(int n);
    int a [2];
    if (n == 0) begin a[1] = 7; return 0; end
    a[1] = 3;
    void'(rec(0));
    return a[1];
  endfunction

  // An automatic function's array is new on every call.
  function automatic int fresh(bit wr);
    int m [2];
    if (wr) m[0] = 9;
    return m[0];
  endfunction

  int s;
  initial begin
    check("fixed array, 1st call", count(1), 1);
    check("fixed array, 2nd call", count(0), 2);
    void'(grid(1, 33));
    check("2-D array", grid(0, 0), 33);
    void'(queue_len(1));
    check("queue", queue_len(2), 2);
    void'(assoc_num("x"));
    check("associative array", assoc_num("y"), 2);
    sum_task(0, s);
    sum_task(1, s);
    check("static task", s, 3);
    void'(explicit_static());
    check("static array of an automatic function", explicit_static(), 12);
    check("recursion", rec(1), 7);
    void'(fresh(1));
    check("automatic function", fresh(0), 0);
    if (errors == 0) $display("ALL PASS"); else $display("%0d FAILED", errors);
  end
endmodule
