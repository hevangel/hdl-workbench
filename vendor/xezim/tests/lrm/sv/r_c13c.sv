// top: r13c
module r13c;
  typedef struct { int a; int b; } ab_t;
  ab_t sab;
  int arr[4];
  logic [7:0] pk;
  task automatic set_ref(ref int x, input int v); x = v; endtask
  function automatic int popc(input int unsigned v); int c = 0; for (int i = 0; i < 32; i++) c += v[i]; return c; endfunction
  localparam PC = popc(32'hF0F0);
  function int acc(input int v); static int s = 0; s += v; return s; endfunction
  task automatic two_out(output logic [3:0] a, output logic [3:0] b); a = 4'h1; b = 4'h2; endtask
  function automatic void modv(int d[]); d[0] = 99; endfunction
  initial begin
    set_ref(sab.b, 7); set_ref(arr[2], 8); $display("T|r1|%0d %0d", sab.b, arr[2]);
    $display("T|r2|%0d", PC);
    $display("T|r3|%0d %0d", acc(1), acc(2));
    two_out(pk[7:4], pk[3:0]); $display("T|r4|%h", pk);
    begin int d[]; d = '{1, 2}; modv(d); $display("T|r5|%0d", d[0]); end
  end
endmodule
