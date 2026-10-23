// Struct formals with a queue member handed on between subroutines.
module p9;
  typedef struct { int x; int ql[$]; } st_t;
  st_t g;
  task automatic grow(ref st_t v);
    v.ql.push_back(v.ql.size());
  endtask
  task automatic outer(ref st_t v);
    grow(v);
    grow(v);
  endtask
  function automatic int peek(input st_t v);
    v.ql.push_back(99);
    return v.ql.size();
  endfunction
  function automatic int depth(input st_t v, input int n);
    if (n == 0) return v.ql.size();
    v.ql.push_back(n);
    return depth(v, n - 1) + 100 * v.ql.size();
  endfunction
  task automatic fill(output st_t v);
    v.x = 3;
    v.ql = {1, 2, 3};
  endtask
  initial begin
    outer(g);
    $display("T|outer sz=%0d e1=%0d", g.ql.size(), g.ql[1]);
    $display("T|peek %0d gsz=%0d", peek(g), g.ql.size());
    $display("T|depth %0d gsz=%0d", depth(g, 3), g.ql.size());
    fill(g);
    $display("T|fill x=%0d sz=%0d e2=%0d", g.x, g.ql.size(), g.ql[2]);
    $finish;
  end
endmodule
