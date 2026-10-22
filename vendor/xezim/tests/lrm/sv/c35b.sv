// top: c35b
module c35b;
  typedef struct packed { byte a, b, c, d; } ps_t;
  import "DPI-C" function int unsigned u32(input int unsigned a);
  import "DPI-C" function shortint s16(input shortint a);
  import "DPI-C" function byte unsigned b8(input byte unsigned a);
  import "DPI-C" function void pstruct(input ps_t i, output ps_t o);
  import "DPI-C" function int arr2d(input int a[][]);
  import "DPI-C" function int open_bits(input bit [7:0] a[]);
  import "DPI-C" function void open_logic(input logic [3:0] a[]);
  import "DPI-C" function int open_size(input int a[]);
  import "DPI-C" context function int call_out();
  import "DPI-C" context function int where();
  import "DPI-C" function void dyn_out(output int a[]);
  import "DPI-C" function int scal_logic(input logic l);
  export "DPI-C" function sv_out;
  export "DPI-C" function sv_where;
  function void sv_out(input int a, output int o, input string s); o = a * 3 + s.len(); endfunction
  function int sv_where(); $display("T|35.7|export %%m=%m"); return 1; endfunction
  ps_t pi, po; int a2[2][3]; bit [7:0] ob[4]; logic [3:0] ol[2]; int os[7:4]; int dy[3];
  initial begin
    $display("T|35.5|u32 %0d s16 %0d b8 %0d", u32(32'hFFFF_FFFE), s16(-32768), b8(8'h0f));
    pi = '{1, 2, 3, 4}; pstruct(pi, po); $display("T|35.5.6|packed struct %h", po);
    foreach (a2[i, j]) a2[i][j] = i * 3 + j; $display("T|35.5.6.1|2d open %0d", arr2d(a2));
    ob = '{1, 2, 3, 250}; $display("T|35.5.6.1|open bits %0d", open_bits(ob));
    ol = '{4'b10xz, 4'b0101}; open_logic(ol);
    $display("T|35.5.6.1|open size/left/right %0d", open_size(os));
    $display("T|35.7|export with output arg %0d", call_out());
    void'(where());
    dyn_out(dy); $display("T|35.5.6.1|open output %p", dy);
    $display("T|35.5.6|svLogic x=%0d z=%0d 1=%0d", scal_logic(1'bx), scal_logic(1'bz), scal_logic(1));
  end
endmodule
