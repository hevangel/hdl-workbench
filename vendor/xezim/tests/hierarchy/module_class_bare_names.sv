module mid #(parameter int K = 1);
  int mv = 10 * K;
  function automatic int mid_fn(); return 3 * K; endfunction
  task automatic mid_t(output int o); o = 5 * K; endtask
  function automatic string mid_m(); return $sformatf("%m"); endfunction
  class MC;
    int o;
    function int b(); return mid_fn(); endfunction
    function int v(); return mv; endfunction
    function void w(); mv = mv + 1; endfunction
    task tk(); mid_t(o); endtask
    function string m(); return mid_m(); endfunction
    static function int sb(); return mid_fn(); endfunction
  endclass
  MC c;
  initial begin
    c = new;
    #1 $display("T|b_%0d|%0d", K, c.b());
    $display("T|v_%0d|%0d", K, c.v());
    c.w(); $display("T|w_%0d|%0d", K, mv);
    c.tk(); $display("T|tk_%0d|%0d", K, c.o);
    $display("T|m_%0d|%s", K, c.m());
    $display("T|sb_%0d|%0d", K, MC::sb());
  end
endmodule
module tb;
  mid #(1) m1();
  mid #(2) m2();
  int mv = 1000;
  function automatic int mid_fn(); return 100; endfunction
  task automatic mid_t(output int o); o = 500; endtask
  function automatic string mid_m(); return "TOP"; endfunction
  initial #5 $finish;
endmodule
