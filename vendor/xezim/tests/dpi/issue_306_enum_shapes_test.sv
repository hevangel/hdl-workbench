// Issue #306 neighbours: enum-typed DPI prototypes (§35.5.6: an enum is its
// base type). The expected values are the reference simulator's, loading this
// same C file. A packed base type is a legal formal but not a legal result
// (§35.5.5), so those enums appear only as arguments.
package e306_pkg;
  typedef enum int { C0, C1, C2 } pe_t;
endpackage

module e306_enum;
  typedef enum int { RED = 1, GREEN = 2, BLUE = 3 } color_e;
  typedef enum bit [7:0] { B0 = 8'h10, B1 = 8'h20, B2 = 8'h30 } be_t;
  typedef enum logic [3:0] { L0 = 4'h1, L1 = 4'h2 } le_t;
  typedef enum byte { YM = 8'shFD, YN = 8'shFE, Y0 = 5, Y1 = 6 } ye_t;
  typedef enum logic { S0, S1 } se_t;

  import "DPI-C" function color_e e306_next(input color_e c);
  import "DPI-C" function void e306_out(output color_e o, inout color_e io);
  import "DPI-C" function e306_pkg::pe_t e306_pkg(input e306_pkg::pe_t c);
  import "DPI-C" function ye_t e306_byte(input ye_t x);
  import "DPI-C" function se_t e306_scalar(input se_t x);
  import "DPI-C" function int e306_bv_in(input be_t x);
  import "DPI-C" function void e306_bv_out(output be_t o, inout be_t io);
  import "DPI-C" function void e306_lv_in(input le_t x, output int a, output int b);
  import "DPI-C" function void e306_lv_out(output le_t o, inout le_t io);

  int f = 0;
  `define CK(e, m) if (!(e)) begin f++; $display("FAIL %s", m); end
  color_e c, c2; e306_pkg::pe_t p; be_t b, b2; le_t l, l2; ye_t y; se_t s; int ia, ib;

  initial begin
    c = e306_next(GREEN); `CK(c == BLUE && c.name() == "BLUE", "int enum result")
    `CK(e306_next(BLUE) == RED, "int enum result in an expression")
    `CK($bits(e306_next(RED)) == 32, "int enum result width")
    c2 = RED; e306_out(c, c2); `CK(c == BLUE && c2 == GREEN, "int enum output / inout")
    p = e306_pkg(e306_pkg::C0); `CK(p == e306_pkg::C1, "package enum")
    y = e306_byte(YM); `CK(y == YN && y.name() == "YN", "byte enum result")
    `CK(e306_byte(Y0) == 6, "byte enum result in an expression")
    s = e306_scalar(S0); `CK(s === S1, "logic enum result")
    s = se_t'(1'bx); s = e306_scalar(s); `CK(s === 1'bx, "logic enum x round trip")
    `CK(e306_bv_in(B2) == 32'h30, "bit [7:0] enum input")
    b2 = B0; e306_bv_out(b, b2); `CK(b == B1 && b2 == B1, "bit [7:0] enum output / inout")
    e306_lv_in(le_t'(4'b1x0z), ia, ib); `CK(ia == 32'hC && ib == 32'h5, "logic [3:0] enum input x/z")
    l2 = L0; e306_lv_out(l, l2); `CK(l == L1 && l2 == L1, "logic [3:0] enum output / inout")
    if (f == 0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
