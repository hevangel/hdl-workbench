module leaf;
  string inst_path = $sformatf("%m");
  function automatic string ip(); return inst_path; endfunction
  function automatic int twice(int x); return 2 * x; endfunction
endmodule
module mid; leaf u_h(); endmodule
class C;
  int q[$];
  function new(); q.push_back(tb.m.u_h.twice(4)); endfunction
  function string nested(); return $sformatf("<%s>", tb.m.u_h.ip()); endfunction
  function int nested2(); return tb.m.u_h.twice(tb.m.u_h.twice(3)); endfunction
  function int loop();
    int s = 0;
    foreach (q[i]) s += tb.m.u_h.twice(q[i]);
    for (int k = 0; k < 3; k++) s += tb.m.u_h.twice(k);
    return s;
  endfunction
  function bit cmp(); return tb.m.u_h.ip() == "tb.m.u_h"; endfunction
  function int len(); return tb.m.u_h.ip().len(); endfunction
endclass
module tb;
  mid m();
  initial begin
    C c = new;
    #1;
    $display("T|ctor|%0d", c.q[0]);
    $display("T|nested|%s", c.nested());
    $display("T|nested2|%0d", c.nested2());
    $display("T|loop|%0d", c.loop());
    $display("T|cmp|%0d", c.cmp());
    $display("T|len|%0d", c.len());
    $finish;
  end
endmodule
