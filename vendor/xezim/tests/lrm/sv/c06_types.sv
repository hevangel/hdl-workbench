// top: c06
module c06;
  // 6.5 nets and variables
  wire [3:0] w1;
  assign w1 = 4'ha;
  // 6.6.1 wire/tri multiple drivers -> x
  wire wd; assign wd = 1'b1; assign wd = 1'b0;
  wire wz; assign wz = 1'bz; assign wz = 1'b1;
  // wand/wor
  wand wa; assign wa = 1'b1; assign wa = 1'b0;
  wor wo; assign wo = 1'b1; assign wo = 1'b0;
  // tri0/tri1 undriven
  tri0 t0; tri1 t1;
  // triand/trior
  triand ta; assign ta = 1'b1; assign ta = 1'bz;
  // trireg
  trireg tr; reg trd; reg tren; assign tr = tren ? trd : 1'bz;
  // supply
  supply0 s0; supply1 s1;
  // uwire single driver
  wire uw; assign uw = 1'b1;
  // 6.7 net declaration with assignment
  wire [7:0] nda = 8'd12;
  // 6.8 variable declarations initial values
  int iv = 5;
  logic [3:0] lv;           // default x
  bit [3:0] bv;             // default 0
  integer intg;             // default x
  byte by = 8'hff;
  shortint shi = -1;
  longint li = 64'h8000_0000_0000_0000;
  // 6.10 implicit nets
  assign impl_n = 1'b1;
  // 6.11 integer types signedness
  // 6.12 real
  real rr = 1.25;
  shortreal sr = 0.1;
  realtime rt = 2.5;
  // 6.14 chandle
  chandle ch;
  // 6.16 strings
  string str = "Hello";
  // 6.17 events
  event e1, e2;
  // 6.18 typedef
  typedef logic [5:0] u6_t;
  u6_t u6 = 6'h3f;
  // 6.19 enums
  typedef enum {RED, GREEN=5, BLUE} color_t;
  typedef enum logic [1:0] {S0, S1, S2} st_t;
  typedef enum {A[3]} aa_t;     // A0 A1 A2
  typedef enum {B[2:4]=10} bb_t;   // B2=10 B3 B4
  color_t col;
  st_t st;
  // 6.20 parameters
  parameter int P1 = 3;
  parameter P2 = 4'b1010;
  parameter [7:0] P3 = 300;
  parameter real PR = 2.5;
  localparam LP = P1 * 2;
  parameter type PT = shortint;
  parameter PS = "abc";
  specparam SP = 7;
  PT ptv;
  // 6.21 lifetimes
  function automatic int fa(); int k = 0; k++; return k; endfunction
  function int fs(); static int k = 0; k++; return k; endfunction
  // 6.24 casting
  typedef struct packed {logic [3:0] hi; logic [3:0] lo;} pk_t;
  pk_t pk;
  initial begin
    #1;
    $display("T|6.5|%h", w1);
    $display("T|6.6.1|wd=%b wz=%b", wd, wz);
    $display("T|6.6.3|wa=%b wo=%b", wa, wo);
    $display("T|6.6.4|t0=%b t1=%b ta=%b", t0, t1, ta);
    $display("T|6.6.6|s0=%b s1=%b uw=%b", s0, s1, uw);
    trd = 1; tren = 1; #1 tren = 0; #1 $display("T|6.6.4tr|tr=%b", tr);
    $display("T|6.7|nda=%0d", nda);
    $display("T|6.8|iv=%0d lv=%b bv=%b intg=%0d by=%0d shi=%0d li=%0d", iv, lv, bv, intg, by, shi, li);
    $display("T|6.10|impl=%b bits=%0d", impl_n, $bits(impl_n));
    $display("T|6.11a|%0d %0d %0d %0d", $bits(byte), $bits(shortint), $bits(int), $bits(longint));
    $display("T|6.11b|%0d %0d", $bits(integer), $bits(time));
    begin byte b2 = 8'h80; int unsigned uu = -1; bit signed [3:0] sb = 4'b1000;
      $display("T|6.11c|%0d %0d %0d", b2, uu, sb); end
    begin int i1 = 32'hffff_ffff; integer i2; i2 = 'hx; $display("T|6.11d|%0d %0d", i1, i2); end
    $display("T|6.12a|%f %f %f", rr, sr, rt);
    $display("T|6.12b|%0d", $bits(rr));
    $display("T|6.14|ch=null:%0d", ch == null);
    // 6.16 string methods
    $display("T|6.16a|len=%0d", str.len());
    str.putc(0, "J"); $display("T|6.16b|%s %c", str, str.getc(1));
    $display("T|6.16c|%s %s", str.toupper(), str.tolower());
    $display("T|6.16d|%0d %0d %0d", str.compare("Jello"), str.compare("Zz") < 0, str.icompare("JELLO"));
    $display("T|6.16e|%s|%s", str.substr(1, 3), str.substr(3, 10));
    begin string n = "123"; string h = "ff"; string o = "17"; string b = "101"; string rs = "3.5e1";
      $display("T|6.16f|%0d %0d %0d %0d %f", n.atoi(), h.atohex(), o.atooct(), b.atobin(), rs.atoreal()); end
    begin string t; t.itoa(-42); $display("T|6.16g|%s", t);
      t.hextoa(255); $display("T|6.16h|%s", t);
      t.octtoa(8); $display("T|6.16i|%s", t);
      t.bintoa(5); $display("T|6.16j|%s", t);
      t.realtoa(1.5); $display("T|6.16k|%s", t); end
    begin string a = "abc", b = "abd";
      $display("T|6.16l|%0d %0d %0d %0d", a < b, a == "abc", a != b, a > b);
      $display("T|6.16m|%s", {a, "-", b});
      $display("T|6.16n|%s", {3{a}});
      $display("T|6.16o|%c", a[1]);
      a[1] = "X"; $display("T|6.16p|%s", a);
      a = ""; $display("T|6.16q|%0d", a.len());
      a = "abc"; a.putc(10, "Z"); $display("T|6.16r|%s", a);
      $display("T|6.16s|%0d", a.getc(10));
      a = "ab"; a = {a, 8'd0, "c"}; $display("T|6.16t|%0d", a.len());
      begin string sv; logic [31:0] l32 = "abcd"; sv = string'(l32); $display("T|6.16u|%s", sv); end
    end
    begin string ns = "  12abc"; $display("T|6.16v|%0d", ns.atoi()); end
    // 6.17 events
    fork
      begin @e1 $display("T|6.17a|e1 at %0t", $time); end
      begin #1 -> e1; end
    join
    e2 = e1; // event alias
    fork
      begin @e2 $display("T|6.17b|e2 alias fired"); end
      begin #1 -> e1; end
    join
    $display("T|6.17c|null=%0d", e2 == e1);
    // 6.18
    $display("T|6.18|%h %0d", u6, $bits(u6_t));
    // 6.19 enum
    col = col.first(); $display("T|6.19a|%s %0d", col.name(), col);
    col = col.next(); $display("T|6.19b|%s %0d", col.name(), col);
    col = col.next(); $display("T|6.19c|%s %0d", col.name(), col);
    col = col.next(); $display("T|6.19d|%s %0d wrap", col.name(), col);
    col = col.prev(); $display("T|6.19e|%s", col.name());
    col = col.last(); $display("T|6.19f|%s num=%0d", col.name(), col.num());
    col = col.next(2); $display("T|6.19g|%s", col.name());
    col = color_t'(3); $display("T|6.19h|[%s] %0d", col.name(), col);
    col = col.next(); $display("T|6.19i|[%s] %0d", col.name(), col);
    $display("T|6.19j|%0d %0d %0d", A0, A2, B3);
    st = S2; $display("T|6.19k|%b %s %0d", st, st.name(), $bits(st));
    $display("T|6.19l|%0d", st + 1);
    begin aa_t ae = A1; bb_t be = B4; $display("T|6.19m|%s %s %0d", ae.name(), be.name(), be); end
    // 6.20
    $display("T|6.20a|%0d %b %0d %0d %f", P1, P2, $bits(P2), P3, PR);
    $display("T|6.20b|%0d %0d %s %0d", LP, $bits(ptv), PS, SP);
    $display("T|6.20c|%0d", $bits(PS));
    // 6.21
    $display("T|6.21|%0d %0d %0d %0d", fa(), fa(), fs(), fs());
    // 6.22 type compatibility
    // 6.23 type operator
    begin var type(iv) tv = 9; $display("T|6.23|%0d %0d", tv, $bits(type(li)));
      if (type(iv) == type(int)) $display("T|6.23b|eq"); else $display("T|6.23b|ne"); end
    // 6.24.1 static casts
    $display("T|6.24.1a|%h", 4'(8'hab));
    $display("T|6.24.1b|%0d", signed'(4'b1111));
    $display("T|6.24.1c|%0d", unsigned'(-1));
    $display("T|6.24.1d|%0d", int'(2.7));
    $display("T|6.24.1e|%f", real'(3));
    $display("T|6.24.1f|%h", 12'(-4'sd1));
    $display("T|6.24.1g|%0d", shortint'(70000));
    $display("T|6.24.1h|%s", color_t'(5) == GREEN ? "y" : "n");
    $display("T|6.24.1i|%b", logic'(2'b10));
    // 6.24.2 $cast
    begin color_t c2; int ok;
      ok = $cast(c2, 6); $display("T|6.24.2a|%0d %s", ok, c2.name());
      ok = $cast(c2, 7); $display("T|6.24.2b|%0d %s", ok, c2.name());
      $cast(c2, 0); $display("T|6.24.2c|%s", c2.name());
    end
    // 6.24.3 bit-stream
    pk = 8'h5a; $display("T|6.24.3a|%h %h", pk.hi, pk.lo);
    begin typedef byte barr_t[4]; barr_t ba; int x32;
      ba = barr_t'(32'h01020304); $display("T|6.24.3b|%p", ba);
      x32 = int'(ba); $display("T|6.24.3c|%h", x32);
      begin typedef bit [7:0] q8_t[$]; q8_t qq; qq = q8_t'(24'habcdef); $display("T|6.24.3d|%p", qq); end
    end
  end
  // 6.21 static initialisation of variables in static block
  initial begin : sblk
    for (int k = 0; k < 2; k++) begin
      static int auto_k = 10;
      auto_k++;
      $display("T|6.21b|%0d", auto_k);
    end
  end
endmodule
