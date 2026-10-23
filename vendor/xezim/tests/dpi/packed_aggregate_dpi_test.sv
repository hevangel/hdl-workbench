// IEEE 1800-2023 §35.5.6 / Annex H: packed struct and union formals (every
// direction, 2- and 4-state, wider than 32 bits), an enum formal,
// multi-dimensional open arrays in and out, and output / inout formals of
// exported functions and tasks. Expected values come from the reference
// simulator. C side: packed_aggregate_dpi.c.
module packed_aggregate_dpi_test;
  typedef struct packed { byte a, b, c, d; } ps_t;
  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } pl_t;
  typedef struct packed { bit [31:0] x; bit [15:0] y; bit [7:0] z; } pw_t;
  typedef union packed { int i; bit [31:0] b; } pu_t;
  typedef enum int { E0 = 3, E1 = 9 } en_t;
  import "DPI-C" function void pstruct(input ps_t i, output ps_t o);
  import "DPI-C" function int pstruct_in(input ps_t i);
  import "DPI-C" function void ps_io(inout ps_t s);
  import "DPI-C" function void ps_out(output ps_t s);
  import "DPI-C" function int pl_in(input pl_t s);
  import "DPI-C" function void pl_out(output pl_t s);
  import "DPI-C" function void pw_inc(input pw_t i, output pw_t o);
  import "DPI-C" function int pu_in(input pu_t u);
  import "DPI-C" function void pu_out(output pu_t u);
  import "DPI-C" function int en_in(input en_t e);
  import "DPI-C" function int arr2d(input int a[][]);
  import "DPI-C" function int arr2d_plain(input int a[][]);
  import "DPI-C" function int arr3d(input byte a[][][]);
  import "DPI-C" function void arr2d_out(output int a[][]);
  import "DPI-C" function int arr2d_fixed(input int a[2][3]);
  import "DPI-C" context function int call_out();
  import "DPI-C" context function int call_outs();
  import "DPI-C" context task call_tout(output int r);
  export "DPI-C" function sv_out;
  export "DPI-C" function sv_o64;
  export "DPI-C" function sv_oreal;
  export "DPI-C" function sv_inout;
  export "DPI-C" task sv_tout;
  function void sv_out(input int a, output int o, input string s); o = a * 3 + s.len(); endfunction
  function void sv_o64(input int a, output longint o); o = 64'h1_0000_0000 + a; endfunction
  function void sv_oreal(output real r); r = 2.5; endfunction
  function void sv_inout(inout int v); v = v * 2 + 1; endfunction
  task sv_tout(output int o); o = 33; endtask
  ps_t pi, po, s; pl_t l; pw_t wi, wo; pu_t u;
  int a2[2][3]; byte a3[2][2][2]; int o2[2][2];
  initial begin
    pi = '{1, 2, 3, 4}; pstruct(pi, po);
    $display("T|ps out=%h in=%h", po, pstruct_in(pi));
    s = '{1, 2, 3, 4}; ps_io(s); $display("T|ps inout=%h", s);
    ps_out(s); $display("T|ps output=%h", s);
    l = 8'b10x1_0110; $display("T|pl in=%0d", pl_in(l));
    pl_out(l); $display("T|pl output=%b", l);
    wi = '{32'hdeadbeef, 16'h1234, 8'h56}; pw_inc(wi, wo); $display("T|pw output=%h", wo);
    u.i = 77; $display("T|pu in=%0d", pu_in(u));
    pu_out(u); $display("T|pu output=%h", u);
    $display("T|enum in=%0d", en_in(E1));
    foreach (a2[i, j]) a2[i][j] = i * 3 + j;
    $display("T|2d %0d %0d %0d", arr2d(a2), arr2d_plain(a2), arr2d_fixed(a2));
    foreach (a3[i, j, k]) a3[i][j][k] = i * 4 + j * 2 + k;
    $display("T|3d %0d", arr3d(a3));
    arr2d_out(o2); $display("T|2d output=%p", o2);
    $display("T|export output=%0d", call_out());
    $display("T|export outputs=%0d", call_outs());
    begin int r; call_tout(r); $display("T|export task output=%0d", r); end
  end
endmodule
