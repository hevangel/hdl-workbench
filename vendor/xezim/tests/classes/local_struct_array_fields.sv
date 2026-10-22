// §7.2.1/§7.4: field reads and writes on elements of packed-struct
// collections local to a function, an automatic task and a block.
module f296;
  typedef struct packed { logic [7:0] d; logic v; } s_t;
  function automatic void f();
    s_t a [];
    s_t q [$];
    s_t fx [3];
    a = new[3];
    a[1].d = 5; a[1].v = 1;
    q.push_back('0); q[0].d = 6;
    fx[2].d = 7;
    $display("T|f %0d %0d %0d %0d", a[1].d, a[1].v, q[0].d, fx[2].d);
    foreach (a[i]) begin a[i].d = i + 1; a[i].v = 1; end
    $display("T|f chk=%0d", std::randomize() with { foreach (a[i]) { a[i].d == i + 1; a[i].v == 1; } });
  endfunction
  task automatic t();
    s_t a [];
    s_t q [$];
    a = new[3];
    a[2].d = 9;
    q.push_back('0); q.push_back('0); q[1].v = 1; q[1].d = 8'hAB;
    $display("T|t %0d %0d %0d", a[2].d, q[1].v, q[1].d);
    $display("T|t chk=%0d", std::randomize() with { a[2].d == 9; q[1].d == 8'hAB; });
  endtask
  initial begin
    s_t b [];
    b = new[2];
    b[1].d = 4;
    $display("T|i %0d", b[1].d);
    f(); t(); t();
  end
endmodule
