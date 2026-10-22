// top: c11x
module c11x;
  logic [7:0] a8, b8, r8;
  logic [15:0] r16;
  logic [3:0] a4;
  logic signed [7:0] sa, sb;
  logic signed [15:0] s16;
  int i;
  integer ig;
  logic [31:0] u32;
  logic [63:0] u64;
  initial begin
    // 11.3.2 precedence
    $display("T|11.3.2a|%0d %0d %0d", 2 + 3 * 4, (2 + 3) * 4, 2 ** 3 ** 2);
    $display("T|11.3.2b|%0d %0d", -2 ** 2, 1 << 2 + 1);
    $display("T|11.3.2c|%b %b", 1 | 0 & 0, 4'b1100 ^ 4'b1010 & 4'b0110);
    $display("T|11.3.2d|%0d", 1 ? 2 : 0 ? 3 : 4);
    $display("T|11.3.2e|%b", 3 < 5 == 1);
    $display("T|11.3.2f|%0d", !1 + 1);
    // 11.6.1 expression sizing in context
    a8 = 8'd200; b8 = 8'd100;
    r16 = a8 + b8; $display("T|11.6.1a|%0d", r16);
    r8 = (a8 + b8) / 2; $display("T|11.6.1b|%0d", r8);
    r16 = (a8 + b8) / 2; $display("T|11.6.1c|%0d", r16);
    i = a8 + b8; $display("T|11.6.1d|%0d", i);
    if (a8 + b8 > 255) $display("T|11.6.1e|wide"); else $display("T|11.6.1e|narrow");
    if ((a8 + b8) > 9'd255) $display("T|11.6.1f|wide"); else $display("T|11.6.1f|narrow");
    $display("T|11.6.1g|%0d", a8 + b8);
    $display("T|11.6.1h|%0d", {a8 + b8});
    a4 = 4'hf;
    r8 = ~a4; $display("T|11.6.1i|%h", r8);         // ~ evaluated at 8 bits: f0
    r8 = {~a4}; $display("T|11.6.1j|%h", r8);       // 00 f? -> 0
    r8 = ~a4 >> 2; $display("T|11.6.1k|%h", r8);
    r8 = a4 << 2; $display("T|11.6.1l|%h", r8);
    r8 = (a4 << 2) >> 2; $display("T|11.6.1m|%h", r8);
    r8 = -a4; $display("T|11.6.1n|%h", r8);
    r8 = a4 * a4; $display("T|11.6.1o|%h", r8);
    $display("T|11.6.1p|%b", a4 == ~4'h0);
    $display("T|11.6.1q|%b", a4 == ~4'h0 + 0);       // ~4'h0 extended to 32 -> ffffffff != f
    $display("T|11.6.1r|%b", &(a4 << 1));
    u64 = 32'hffff_ffff + 1; $display("T|11.6.1s|%h", u64);
    u64 = 32'hffff_ffff + 64'd1; $display("T|11.6.1t|%h", u64);
    u64 = 1 << 40; $display("T|11.6.1u|%h", u64);
    u64 = 64'd1 << 40; $display("T|11.6.1v|%h", u64);
    u32 = 1 << 31; $display("T|11.6.1w|%0d", u32);
    // conditional operand widths
    r16 = 1 ? a8 : 16'h0; $display("T|11.6.1x|%h", r16);
    r16 = a4 ? a8 + b8 : 0; $display("T|11.6.1y|%0d", r16);
    // 11.8.1 signedness rules
    sa = -8'sd10; sb = 8'sd3;
    s16 = sa; $display("T|11.8.1a|%0d", s16);
    s16 = sa + a8; $display("T|11.8.1b|%0d", s16);             // unsigned context
    s16 = sa / sb; $display("T|11.8.1c|%0d", s16);
    s16 = sa % sb; $display("T|11.8.1d|%0d", s16);
    s16 = sa >>> 1; $display("T|11.8.1e|%0d", s16);
    s16 = $unsigned(sa) >>> 1; $display("T|11.8.1f|%0d", s16);
    s16 = sa[7:0]; $display("T|11.8.1g|%0d", s16);
    s16 = {sa}; $display("T|11.8.1h|%0d", s16);
    s16 = sa * 2'sb11; $display("T|11.8.1i|%0d", s16);
    s16 = sa * 2'b11; $display("T|11.8.1j|%0d", s16);
    $display("T|11.8.1k|%b", sa < 0);
    $display("T|11.8.1l|%b", sa < 8'd0);
    $display("T|11.8.1m|%b", sa < 0.0);
    $display("T|11.8.1n|%0d", sa ** 2);
    $display("T|11.8.1o|%0d", 8'sd2 ** -1);
    $display("T|11.8.1p|%0d", (-8'sd2) ** 3);
    ig = -7; $display("T|11.8.1q|%0d %0d", ig / 2, ig % 2);
    $display("T|11.8.1r|%0d", -7 >>> 1);
    $display("T|11.8.1s|%0d", -32'sd7 >>> 1);
    $display("T|11.8.1t|%0d", 32'd5 - 32'd7);
    $display("T|11.8.1u|%0d", 5 - 7);
    $display("T|11.8.1v|%0d", 3'sb100 + 3'sb100);
    i = 3'sb100; $display("T|11.8.1w|%0d", i);
    i = 3'b100; $display("T|11.8.1x|%0d", i);
    i = $signed(3'b100); $display("T|11.8.1y|%0d", i);
    // 11.4.14.4 streaming with with-clause on arrays
    begin int src[$] = {1, 2, 3, 4}; int dst[$]; logic [63:0] w; w = {>>{src with [0:1]}}; $display("T|11.4.14.4|%h", w); end
    // 11.5.1 indexed part-select with variable base on packed array of structs
    begin logic [31:0] w = 32'hdeadbeef; int b = 8; $display("T|11.5.1g|%h %h", w[b +: 16], w[b+8 -: 8]); w[b +: 8] = 8'h00; $display("T|11.5.1h|%h", w); end
    // assignment to out-of-range part-select
    begin logic [7:0] w = 8'h00; int b = 6; w[b +: 4] = 4'hf; $display("T|11.5.1i|%h", w); end
    // 11.5.3 longest static prefix not observable here
    // 11.4.12.1 replication with zero in concat and parameter
    begin localparam N = 0; logic [3:0] w = {4'h5, {N{1'b1}}}; $display("T|11.4.12.1|%h", w); end
    // 11.4.11 conditional with x condition on vectors merging bits
    begin logic c = 1'bx; logic [3:0] a = 4'b1100, b = 4'b1010; $display("T|11.4.11g|%b", c ? a : b); end
    // 11.4.5 equality with different widths and signedness
    $display("T|11.4.5c|%b %b", 4'sb1111 == 8'sb11111111, 4'sb1111 == 8'b11111111);
    $display("T|11.4.5d|%b", -1 == 32'hffffffff);
    $display("T|11.4.5e|%b", 1'b1 == 2'b01);
    // increments on 4-state with x
    begin logic [3:0] w = 4'b10x0; w++; $display("T|11.4.2c|%b", w); end
    // unary minus of unsigned and real
    $display("T|11.4.3l|%0d %f", -8'd1, -(1.5));
  end
endmodule
