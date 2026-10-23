// top: t35b   (C side: 35.5.6_dpi_misc.c; gcc -shared -fPIC -I <svdpi.h dir>)
module t35b;
  typedef struct packed { byte a, b, c, d; } ps_t;
  import "DPI-C" function void pstruct(input ps_t i, output ps_t o);
  import "DPI-C" function int pstruct_in(input ps_t i);
  import "DPI-C" function int arr2d(input int a[][]);
  import "DPI-C" context function int call_out();
  export "DPI-C" function sv_out;
  function void sv_out(input int a, output int o, input string s); o = a * 3 + s.len(); endfunction
  ps_t pi, po; int a2[2][3];
  initial begin
    pi = '{1, 2, 3, 4}; pstruct(pi, po); $display("T|a|packed struct out=%h in=%h (expect 02030405 01020304)", po, pstruct_in(pi));
    foreach (a2[i, j]) a2[i][j] = i * 3 + j; $display("T|b|2-D open array %0d (expect 152)", arr2d(a2));
    $display("T|c|export output arg %0d (expect 17)", call_out());
  end
endmodule
