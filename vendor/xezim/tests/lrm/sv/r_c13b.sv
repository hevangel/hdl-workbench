// top: r13b
module r13b;
  task shared_t(input int v, output int o); int tmp; tmp = v; #2 o = tmp; endtask
  task automatic loop_t(output int n); n = 0; forever begin #1 n++; if (n == 3) return; end endtask
  function automatic int f_out(input int a, output int b); b = a * 2; return a + 1; endfunction
  int o1, o2, r, b;
  initial begin
    fork
      begin shared_t(1, o1); end
      begin #1 shared_t(2, o2); end
    join
    $display("T|r1|o1=%0d o2=%0d", o1, o2);
    r = f_out(3, b) + 10; $display("T|r2|%0d %0d", r, b);
    loop_t(r); $display("T|r3|n=%0d", r);
  end
endmodule
