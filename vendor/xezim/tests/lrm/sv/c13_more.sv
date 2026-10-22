// top: c13x
module c13x;
  int g = 0;
  // 13.3.1 static task: concurrent calls share variables
  task shared_t(input int v, output int o); int tmp; tmp = v; #2 o = tmp; endtask
  // 13.4 function with output argument used in expression
  function automatic int f_out(input int a, output int b); b = a * 2; return a + 1; endfunction
  // 13.4.1 void function
  function automatic void vf(ref int x); x++; endfunction
  // 13.2 task enable with timing, disable task inside itself
  task automatic loop_t(output int n); n = 0; forever begin #1 n++; if (n == 3) return; end endtask
  // 13.4.2 recursive automatic with array arg
  function automatic int rsum(int q[$]); if (q.size() == 0) return 0; return q[0] + rsum(q[1:$]); endfunction
  // 13.5.2 pass by reference of array element and struct member
  typedef struct { int a; int b; } ab_t;
  ab_t sab;
  int arr[4];
  task automatic set_ref(ref int x, input int v); x = v; endtask
  // 13.4.3 constant function using a loop and a case, used for a parameter and in a generate
  function automatic int popc(input int unsigned v); int c = 0; for (int i = 0; i < 32; i++) c += v[i]; return c; endfunction
  localparam PC = popc(32'hF0F0);
  // 13.7 task/function names and hierarchical calls covered in 23
  // 13.4.4 function called in a continuous assign with a static variable inside
  function int acc(input int v); static int s = 0; s += v; return s; endfunction
  // output args to a part select and to a concatenation
  task automatic two_out(output logic [3:0] a, output logic [3:0] b); a = 4'h1; b = 4'h2; endtask
  logic [7:0] pk;
  // default output argument value? (not allowed) - skip
  // 13.5.1 pass by value of dynamic array: callee modifications not visible
  function automatic void modv(int d[]); d[0] = 99; endfunction
  initial begin
    int o1, o2, r, b;
    fork
      begin shared_t(1, o1); end
      begin #1 shared_t(2, o2); end
    join
    $display("T|13.3.1|o1=%0d o2=%0d", o1, o2);   // static task: tmp shared -> o1 = 2
    r = f_out(3, b) + 10; $display("T|13.4b|%0d %0d", r, b);
    g = 5; vf(g); $display("T|13.4.1d|%0d", g);
    loop_t(r); $display("T|13.2a|n=%0d", r);
    $display("T|13.4.2b|%0d", rsum('{1, 2, 3, 4}));
    set_ref(sab.b, 7); set_ref(arr[2], 8); $display("T|13.5.2g|%0d %0d", sab.b, arr[2]);
    $display("T|13.4.3b|%0d", PC);
    $display("T|13.4.4|%0d %0d", acc(1), acc(2));
    two_out(pk[7:4], pk[3:0]); $display("T|13.5.1c|%h", pk);
    begin int d[]; d = '{1, 2}; modv(d); $display("T|13.5.1d|%0d", d[0]); end
  end
endmodule
