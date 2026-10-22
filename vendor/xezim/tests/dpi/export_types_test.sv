// #291: every formal type an exported subroutine takes from C (IEEE 1800
// §35.5.6, Annex H): chandle, string results and outputs, packed vectors
// wider than 64 bits by svLogicVecVal / svBitVecVal pointer, byte / shortint /
// shortreal / bit / logic in their C types, unpacked structs and arrays by
// pointer, and an exported task with chandle and wide formals. Expected
// values from the reference simulator.
module top;
  typedef struct { int a; byte b; real r; string s; chandle h; } us_t;
  typedef struct { shortint s; logic [7:0] v; bit [69:0] w; logic l; int arr[3]; } um_t;
  typedef struct { int x; byte y; } s1_t;
  import "DPI-C" context function void c_main();
  import "DPI-C" context task c_tmain();
  function void put(string s); $display("T|%s", s); endfunction
  export "DPI-C" function put;
  // chandle in/out/inout/result
  function chandle f_ch(input chandle h, output chandle o, inout chandle io);
    o = io; io = h; return h;
  endfunction
  export "DPI-C" function f_ch;
  // string result/out/inout
  function string f_str(input string s, output string o, inout string io);
    o = {s, "-out"}; io = {io, "+", s}; return {"r:", s};
  endfunction
  export "DPI-C" function f_str;
  // wide 4-state in/out/inout
  function void f_l(input logic [64:0] a, input logic [99:0] b, output logic [127:0] o, inout logic [70:0] io);
    $display("T|f_l a=%h b=%h io=%h", a, b, io);
    o = {a[63:0], b[63:0]}; io = io + 1;
  endfunction
  export "DPI-C" function f_l;
  // wide 2-state
  function void f_b(input bit [99:0] a, output bit [65:0] o, inout bit [127:0] io);
    $display("T|f_b a=%h io=%h", a, io);
    o = a[65:0]; io = ~io;
  endfunction
  export "DPI-C" function f_b;
  // small scalars
  function byte f_sc(input byte a, input shortint b, input shortreal c, input bit d, input logic e,
                     output byte oa, output shortint ob, output shortreal oc, output bit od, output logic oe);
    $display("T|f_sc a=%0d b=%0d c=%0.2f d=%b e=%b", a, b, c, d, e);
    oa = a - 1; ob = b * 2; oc = c * 2; od = !d; oe = 1'bz;
    return a + 1;
  endfunction
  export "DPI-C" function f_sc;
  function bit f_rb(input int i); return i[0]; endfunction
  export "DPI-C" function f_rb;
  function logic f_rl(input int i); return i == 2 ? 1'bx : (i == 3 ? 1'bz : i[0]); endfunction
  export "DPI-C" function f_rl;
  function shortreal f_rf(input shortreal x); return x + 0.5; endfunction
  export "DPI-C" function f_rf;
  // unpacked structs
  function int f_us(input us_t a, output us_t o, inout um_t m);
    $display("T|f_us a=%0d %0d %0.1f %s %0d", a.a, a.b, a.r, a.s, a.h == null);
    $display("T|f_us m=%0d %h %h %b %0d %0d %0d", m.s, m.v, m.w, m.l, m.arr[0], m.arr[1], m.arr[2]);
    o.a = a.a * 2; o.b = -3; o.r = 2.25; o.s = "struct-out"; o.h = a.h;
    m.s = m.s + 1; m.v = 8'h5a; m.w = ~m.w; m.l = 1'bx; m.arr[1] = 77;
    return a.a + a.b;
  endfunction
  export "DPI-C" function f_us;
  // unpacked arrays
  function void f_ua(input int a[4], output int o[0:2], inout byte io[3:0], input logic [7:0] lv[2], input s1_t sa[2], output s1_t so[2]);
    $display("T|f_ua a=%0d %0d %0d %0d", a[0], a[1], a[2], a[3]);
    $display("T|f_ua io[3]=%0d io[0]=%0d lv=%h %h sa=%0d/%0d %0d/%0d", io[3], io[0], lv[0], lv[1], sa[0].x, sa[0].y, sa[1].x, sa[1].y);
    foreach (o[i]) o[i] = a[i] + 10;
    io[3] = 33; io[0] = -1;
    so[0].x = 5; so[0].y = 6; so[1].x = sa[0].x + sa[1].x; so[1].y = 9;
  endfunction
  export "DPI-C" function f_ua;
  function int f_2d(input int a[2][3], output real r[2]);
    r[0] = a[0][2] + 0.5; r[1] = a[1][0];
    return a[0][0] + a[1][2];
  endfunction
  export "DPI-C" function f_2d;
  // task with chandle and wide args
  task t_w(input chandle h, input logic [127:0] w, output int o);
    #5; o = (h == null) ? -1 : w[127:96];
    $display("T|t_w t=%0t w=%h", $time, w);
  endtask
  export "DPI-C" task t_w;
  initial begin
    c_main();
    c_tmain();
    $display("T|after c_tmain t=%0t", $time);
    $finish;
  end
endmodule
