// Issue #306 neighbours: x and z through every 4-state DPI shape. The
// expected values are the reference simulator's, loading this same C file;
// they are the svLogic / svLogicVecVal encodings of IEEE 1800 Annex H.
module s306_svlogic;
  typedef logic lg_t;
  import "DPI-C" function void  s306_lout(input int sel, output logic b);
  import "DPI-C" function logic s306_lret(input int sel);
  import "DPI-C" function int   s306_lin(input logic b);
  import "DPI-C" function void  s306_linout(inout logic b, output int seen);
  import "DPI-C" function int   s306_impl(input b, output o);
  import "DPI-C" task           s306_tout(output logic b);
  import "DPI-C" function void  s306_bout(output bit b);
  import "DPI-C" function bit   s306_bret();
  import "DPI-C" s306_lret = function lg_t s306_tret(input int sel);
  import "DPI-C" function void  s306_vout(output logic [7:0] v);
  import "DPI-C" function void  s306_vinout(inout logic [7:0] v, output int a, output int b);
  import "DPI-C" function void  s306_vwide(output logic [39:0] v);
  import "DPI-C" function void  s306_vin(input logic [7:0] v, output int a, output int b);

  int f = 0;
  `define CK(e, m) if (!(e)) begin f++; $display("FAIL %s", m); end
  logic l; lg_t tl; bit b; logic [7:0] v; logic [39:0] w; int a, bb, s;

  initial begin
    s306_lout(0, l); `CK(l === 1'b0, "lout 0")
    s306_lout(1, l); `CK(l === 1'b1, "lout 1")
    s306_lout(2, l); `CK(l === 1'bz, "lout z")
    s306_lout(3, l); `CK(l === 1'bx, "lout x")
    s306_lout(3, tl); `CK(tl === 1'bx, "typedef lout x")
    l = s306_lret(0); `CK(l === 1'b0, "lret 0")
    l = s306_lret(1); `CK(l === 1'b1, "lret 1")
    l = s306_lret(2); `CK(l === 1'bz, "lret z")
    l = s306_lret(3); `CK(l === 1'bx, "lret x")
    `CK(s306_lret(2) === 1'bz, "lret z in an expression")
    `CK($bits(s306_lret(3)) == 1, "lret is one bit")
    tl = s306_tret(3); `CK(tl === 1'bx, "typedef result x")
    `CK(s306_lin(1'b0) == 0 && s306_lin(1'b1) == 1, "lin 0/1")
    `CK(s306_lin(1'bz) == 2 && s306_lin(1'bx) == 3, "lin z/x")
    l = 1'bx; s306_linout(l, s); `CK(s == 3 && l === 1'bz, "inout x in, z out")
    l = 1'bz; s306_linout(l, s); `CK(s == 2 && l === 1'bz, "inout z in, z out")
    l = 1'b0; s = s306_impl(1'bz, l); `CK(s == 2 && l === 1'bx, "implicit formals")
    s306_tout(l); `CK(l === 1'bz, "task output z")
    s306_bout(b); `CK(b === 1'b1, "bit output")
    b = s306_bret(); `CK(b === 1'b1, "bit result")
    s306_vout(v); `CK(v === 8'b1x0z_10zx, "vector output")
    v = 8'b1x0z_10xz; s306_vinout(v, a, bb);
    `CK(a == 32'hCA && bb == 32'h53, "vector inout in")
    `CK(v === 8'bx100_010x, "vector inout out")
    s306_vwide(w); `CK(w === 40'bzxzx1010_1111111111111111_xxxxxxxxxxxxxxxx, "wide vector output")
    s306_vin(8'b01xz_zx10, a, bb); `CK(a == 32'h66 && bb == 32'h3C, "vector input x/z")
    if (f == 0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
