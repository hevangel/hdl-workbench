module tb;
  import "DPI-C" function byte      tm_byte(input byte x);
  import "DPI-C" function shortint  tm_short(input shortint x);
  import "DPI-C" function int       tm_int(input int x);
  import "DPI-C" function longint   tm_long(input longint x);
  import "DPI-C" function real      tm_real(input real x);
  import "DPI-C" function shortreal tm_shortreal(input shortreal x);
  import "DPI-C" function void      tm_out(output int o);
  import "DPI-C" function void      tm_inout(inout int io);
  import "DPI-C" function int       tm_bitvec(input bit [31:0] v);
  import "DPI-C" function int       tm_bitvec64(input bit [63:0] v);
  import "DPI-C" function int       tm_logic_aval(input logic [31:0] v);
  import "DPI-C" function longint   tm_logic_ab(input logic [7:0] v);
  import "DPI-C" function int       tm_strlen(input string s);

  int f = 0; int o; int io; real r; shortreal sr; longint ab;
  `define CK(e,m) if(!(e)) begin f++; $display("FAIL %s", m); end
  initial begin
    `CK(tm_byte(8'sd5)==6,          "byte")
    `CK(tm_short(16'sd5)==6,        "short")
    `CK(tm_int(5)==6,               "int")
    `CK(tm_long(64'd5)==6,          "long")
    r=tm_real(2.5);  `CK(r==5.0,    "real")
    sr=tm_shortreal(2.5); `CK(sr==5.0, "shortreal")
    tm_out(o);       `CK(o==77,     "out")
    io=5; tm_inout(io); `CK(io==105,"inout")
    `CK(tm_bitvec(32'hDEADBEEF)==32'hDEADBEEF, "bitvec32")
    `CK(tm_bitvec64(64'h2_0000_0003)==5,       "bitvec64")
    `CK(tm_logic_aval(32'h000000FF)==255,      "logic_aval")
    // x/z: 8'b1x0z_1x0z -> aval=0xCC, bval=0x55 (0=00,1=10,Z=01,X=11)
    ab = tm_logic_ab(8'b1x0z_1x0z);
    `CK(ab[31:0]==32'h000000CC,  "logic aval x/z")
    `CK(ab[63:32]==32'h00000055, "logic bval x/z")
    `CK(tm_strlen("hello")==5,   "strlen")
    if(f==0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
