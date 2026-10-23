// DPI audit, family B: `integer` / `time` formals are svLogicVecVal* (an
// int by value made C reading the pointer crash), and unpacked structs pass
// as C structs (they were left unbound). The expected values are the
// reference simulator's, loading this same C file.
module dpi_audit_types;
  typedef struct { int a; byte b; real c; logic d; } s1_t;
  typedef struct { shortint h; logic [7:0] v; bit [69:0] w; string s; int arr[3]; } s2_t;
  typedef struct { s1_t in; longint z; } s3_t;
  import "DPI-C" function void aty_integer(input integer i, output integer o, inout integer io,
                                           output int a, output int b);
  import "DPI-C" function void aty_time(input time t, output time o, output int lo, output int hi);
  import "DPI-C" function int aty_s_in(input s1_t s);
  import "DPI-C" function void aty_s_out(output s1_t s);
  import "DPI-C" function void aty_s_io(inout s1_t s);
  import "DPI-C" function int aty_s2_in(input s2_t s);
  import "DPI-C" function void aty_s2_out(output s2_t s);
  import "DPI-C" function longint aty_s3_in(input s3_t s);
  import "DPI-C" function int aty_sa_in(input s1_t a[2]);
  import "DPI-C" function void aty_sa_out(output s1_t a[2]);
  int f = 0;
  `define CK(e, m) if (!(e)) begin f++; $display("FAIL %s", m); end
  integer io, o; time t; int a, b; s1_t u; s2_t u2; s3_t u3; s1_t ua[2];

  task automatic local_struct();
    s1_t l;
    l = '{a: 2, b: 1, c: 0.5, d: 1'b0};
    `CK(aty_s_in(l) == 2010, "automatic struct in")
    aty_s_out(l); `CK(l.a == 11 && l.d === 1'bz, "automatic struct out")
  endtask

  initial begin
    io = 32'h0000_000f;
    aty_integer(32'h0000_00xz, o, io, a, b);
    `CK(a == 32'hf0 && b == 32'hff, "integer in x/z")
    `CK(o === 32'b0001_00xz, "integer out")
    `CK(io === 32'hffff_fff0, "integer inout")
    aty_time(64'h0000_0002_0000_0005, t, a, b);
    `CK(a == 5 && b == 2, "time in")
    `CK(t == 64'h0000_0001_0000_0007, "time out")
    u = '{a: 3, b: -4, c: 5.9, d: 1'bx}; `CK(aty_s_in(u) == 2972, "struct in")
    aty_s_out(u); `CK(u.a == 11 && u.b == -2 && u.c == 2.75 && u.d === 1'bz, "struct out")
    u.d = 1'bx; aty_s_io(u); `CK(u.a == 12 && u.d === 1'b1, "struct inout")
    u2 = '{h: 5, v: 8'h1z, w: 70'h3_0000_0000_0000_0000, s: "abcd", arr: '{1, 2, 3}};
    `CK(aty_s2_in(u2) == 3486, "struct with vectors, string, array in")
    aty_s2_out(u2);
    `CK(u2.h == -9 && u2.v === 8'b1010_zxzx && u2.w == 70'h03_0000_0002_0000_0001, "struct out vectors")
    `CK(u2.s == "c-str" && u2.arr[0] == 4 && u2.arr[1] == 5 && u2.arr[2] == 6, "struct out string, array")
    u3.in = '{a: 9, b: 0, c: 0.0, d: 1'b1}; u3.z = 64'd100; `CK(aty_s3_in(u3) == 109, "nested struct")
    ua[0] = '{a: 1, b: 0, c: 0.0, d: 1'b0}; ua[1] = '{a: 2, b: 0, c: 0.0, d: 1'b0};
    `CK(aty_sa_in(ua) == 12, "struct array in")
    aty_sa_out(ua); `CK(ua[0].a == 70 && ua[1].a == 80 && ua[0].d === 1'bx && ua[1].d === 1'b1, "struct array out")
    local_struct();
    if (f == 0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
