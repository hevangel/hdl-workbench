interface ifc;
  int v = 11;
  function automatic int iget(); return v + 1; endfunction
endinterface
module leaf #(parameter int P = 0);
  int counter = 5;
  event ev;
  function automatic int get_p(); return P; endfunction
  function automatic void fill(ref int q[$]); q.push_back(P); q.push_back(P+1); endfunction
  function automatic int outp(output int o); o = P * 10; return 1; endfunction
  function automatic void show(); $display("T|show_m_%0d|%m", P); endfunction
  task automatic wait_t(output int t); #3; t = $time; endtask
  function automatic int nest(); return get_p() + 100; endfunction
  function automatic string tbref(); return tb.top_s; endfunction
endmodule
module mid;
  leaf #(1) u_h();
  leaf #(2) arr[2] ();
  for (genvar g = 0; g < 2; g++) begin : gen
    leaf #(10+g) u();
  end
endmodule
class PC;
  static function int go(); return tb.m.u_h.get_p() + tb.m.u_h.nest(); endfunction
endclass
class B #(int W = 4);
  function int pf(); return tb.m.u_h.get_p() + W; endfunction
endclass
class C;
  int q[$];
  int o, r, t;
  function void run();
    $display("T|root|%0d", $root.tb.m.u_h.get_p());
    $display("T|gen1|%0d", tb.m.gen[1].u.get_p());
    $display("T|arr1|%0d", tb.m.arr[1].get_p());
    $display("T|iface|%0d", tb.i0.iget());
    tb.m.u_h.fill(q); $display("T|refq|%0d %0d %0d", q.size(), q[0], q[1]);
    r = tb.m.u_h.outp(o); $display("T|outp|%0d %0d", r, o);
    tb.m.u_h.show();
    void'(tb.m.u_h.nest()); $display("T|voidcast|ok");
    $display("T|nest|%0d", tb.m.u_h.nest());
    $display("T|arith|%0d", tb.m.u_h.get_p() * 3 + tb.m.u_h.counter);
    $display("T|tbref|%s", tb.m.u_h.tbref());
    tb.m.u_h.counter = 9; $display("T|wr|%0d", tb.m.u_h.counter);
    $display("T|cond|%0d", (tb.m.u_h.get_p() == 1) ? 7 : 8);
    if (tb.m.u_h.get_p()) $display("T|ifcall|yes"); else $display("T|ifcall|no");
    $display("T|argcall|%0d", tb.m.u_h.outp(o) + o);
  endfunction
  task automatic tk();
    tb.m.u_h.wait_t(t); $display("T|blk_task|%0d", t);
    fork
      begin tb.m.u_h.wait_t(t); $display("T|fork_task|%0d", t); end
    join
    -> tb.m.u_h.ev;
  endtask
endclass
module tb;
  string top_s = "TOPS";
  mid m();
  ifc i0();
  initial begin
    C c; B#(5) b;
    c = new; b = new;
    c.run();
    $display("T|pkg|%0d", PC::go());
    $display("T|param|%0d", b.pf());
    fork
      begin @(m.u_h.ev); $display("T|evt|%0t", $time); end
      c.tk();
    join
    $finish;
  end
endmodule
