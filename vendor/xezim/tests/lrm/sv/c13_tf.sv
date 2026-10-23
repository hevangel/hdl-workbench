// top: c13
module c13;
  // 13.3 static tasks keep locals
  task st_task(output int o); int k; k++; o = k; endtask
  task automatic at_task(output int o); int k; k++; o = k; endtask
  // static function with explicit static var, automatic function
  function int sf(); int c; c = c + 1; return c; endfunction
  function automatic int af(); int c; c = c + 1; return c; endfunction
  // 13.4.1 return value / function name assign
  function [7:0] oldstyle(input [7:0] x); oldstyle = x + 1; endfunction
  function int noreturn(); endfunction
  // 13.4.2 recursion
  function automatic int fact(int n); return n <= 1 ? 1 : n * fact(n - 1); endfunction
  // 13.4.3 constant functions
  function automatic int clog2f(int v); int r = 0; v = v - 1; while (v > 0) begin r++; v >>= 1; end return r; endfunction
  localparam W = clog2f(17);
  logic [W-1:0] wv;
  // 13.5 argument directions
  task automatic dirs(input int i, output int o, inout int io, ref int rf); o = i + 1; io = io * 2; rf = rf + 100; endtask
  function automatic void cref(const ref int arr[4], output int s); s = 0; foreach (arr[k]) s += arr[k]; endfunction
  task automatic refwait(ref logic sig, output time t); @(posedge sig); t = $time; endtask
  // 13.5.3 defaults
  function automatic int defs(int a = 1, int b = a + 1, int c = 10); return a * 100 + b * 10 + c; endfunction
  task automatic tdef(input int a, output int o, input int b = 5); o = a + b; endtask
  // 13.5.4 named binding
  // arrays as args
  function automatic int sumdyn(int d[]); int s = 0; foreach (d[k]) s += d[k]; return s; endfunction
  function automatic void modq(ref int q[$]); q.push_back(99); endfunction
  function automatic int sumq(int q[$]); return q.sum(); endfunction
  function automatic void cpyarr(input int a[3], output int b[3]); b = a; b[0] = -1; endfunction
  // struct return
  typedef struct { int x; int y; } pt_t;
  function automatic pt_t mkpt(int a); pt_t p; p.x = a; p.y = a * 2; return p; endfunction
  // 13.4.4 background processes from function: fork/join_none allowed
  int bgv = 0;
  function automatic void spawn(); fork #3 bgv = 7; join_none endfunction
  // output arg copy-out time
  logic clk = 0;
  task automatic out_late(output int o); o = 1; #2 o = 2; endtask
  // 13.8 parameterized via class static
  class Par #(int N = 2); static function int twice(int v); return v * N; endfunction endclass
  // inout copies at end
  int g = 1;
  task automatic modg(inout int x); x = 5; $display("T|13.5.2c|during g=%0d", g); endtask
  task automatic refg(ref int x); x = 6; $display("T|13.5.2d|during g=%0d", g); endtask
  // void function call discarding; 13.4.1 void'()
  function int sidefx(); bgv++; return 42; endfunction
  initial begin
    int o1, o2, io, rf, s;
    int a4[4] = '{1,2,3,4};
    time tt;
    st_task(o1); st_task(o2); $display("T|13.3a|%0d %0d", o1, o2);
    at_task(o1); at_task(o2); $display("T|13.3b|%0d %0d", o1, o2);
    $display("T|13.4a|%0d %0d %0d %0d", sf(), sf(), af(), af());
    $display("T|13.4.1a|%0d %0d", oldstyle(8'd255), noreturn());
    $display("T|13.4.2|%0d", fact(10));
    $display("T|13.4.3|W=%0d bits=%0d", W, $bits(wv));
    io = 3; rf = 1; dirs(4, o1, io, rf); $display("T|13.5a|%0d %0d %0d", o1, io, rf);
    cref(a4, s); $display("T|13.5.2a|%0d", s);
    fork refwait(clk, tt); #4 clk = 1; join $display("T|13.5.2b|%0t", tt);
    $display("T|13.5.3a|%0d %0d %0d %0d", defs(), defs(2), defs(2, 3), defs(, , 7));
    $display("T|13.5.3b|%0d", defs(.c(1), .a(4)));
    tdef(1, o1); $display("T|13.5.3c|%0d", o1);
    tdef(.b(10), .a(1), .o(o1)); $display("T|13.5.4|%0d", o1);
    begin int d[] = '{1, 2, 3}; int q[$] = {4}; int a3[3] = '{7, 8, 9}; int b3[3];
      $display("T|13.5.5a|%0d %0d", sumdyn(d), sumdyn(a3));
      modq(q); $display("T|13.5.5b|%p %0d", q, sumq(q));
      cpyarr(a3, b3); $display("T|13.5.5c|%p %p", a3, b3); end
    begin pt_t p = mkpt(3); $display("T|13.4.5|%0d %0d %0d", p.x, p.y, mkpt(5).y); end
    spawn(); #4 $display("T|13.4.4|bgv=%0d", bgv);
    fork out_late(o1); join_none #1 $display("T|13.5.1|o1 during=%0d", o1); #2 $display("T|13.5.1b|o1 after=%0d", o1);
    $display("T|13.8|%0d", Par#(5)::twice(3));
    g = 1; modg(g); $display("T|13.5.2e|after g=%0d", g);
    g = 1; refg(g); $display("T|13.5.2f|after g=%0d", g);
    bgv = 0; void'(sidefx()); sidefx(); $display("T|13.4.1b|bgv=%0d", bgv);
    // function in continuous context below
  end
  // 13.4 function called in continuous assign
  wire [7:0] fw = oldstyle(8'd4);
  initial #1 $display("T|13.4.1c|%0d", fw);
endmodule
