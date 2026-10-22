// top: c11
module c11;
  logic [3:0] a, b, x4;
  logic [7:0] r8;
  logic signed [7:0] s8;
  logic signed [3:0] s4;
  int i;
  integer ig;
  real rl;
  logic [15:0] r16;
  // 11.12 let
  let max2(p, q) = (p > q) ? p : q;
  let inrange(v, lo, hi) = v >= lo && v <= hi;
  initial begin
    // 11.4.1 assignment ops covered in c10
    // 11.4.2 increment/decrement
    i = 5; $display("T|11.4.2a|%0d %0d %0d", i++, i, ++i);
    i = 5; i = i++ + 1; $display("T|11.4.2b|%0d", i);
    // 11.4.3 arithmetic
    a = 4'd7; b = 4'd0;
    $display("T|11.4.3a|%b %b", a / b, a % b);   // x
    $display("T|11.4.3b|%0d %0d %0d %0d", -7 / 2, -7 % 2, 7 % -2, -7 % -2);
    $display("T|11.4.3c|%0d %0d %0d", 2 ** 10, (-2) ** 3, 2 ** -1);
    $display("T|11.4.3d|%0d %0d %0d", 0 ** -1, (-1) ** -3, 3 ** 0);
    $display("T|11.4.3e|%b", 4'b10x1 + 4'b0001);
    $display("T|11.4.3f|%0d", 8'sd100 + 8'sd100);   // overflow in 8 bits? self-determined in display -> 32? no: max(8,8)
    s8 = 8'sd100 + 8'sd100; $display("T|11.4.3g|%0d", s8);
    $display("T|11.4.3h|%f %f", 2.0 ** 0.5, 7.0 / 2);
    $display("T|11.4.3i|%0d", -4'd3);
    $display("T|11.4.3j|%0d", 4'd3 - 4'd5);
    // 11.4.4 relational with x and signedness
    $display("T|11.4.4a|%b %b", 4'b1x00 < 4'b1111, -1 < 1);
    $display("T|11.4.4b|%b", -1 < 1'b1);           // unsigned compare
    $display("T|11.4.4c|%b", -4'sd1 < 4'sd1);
    $display("T|11.4.4d|%b", 3'sb111 > 8'd0);      // mixed -> unsigned 3'b111 extended
    // 11.4.5 equality
    $display("T|11.4.5a|%b %b %b %b", 4'b1x01 == 4'b1x01, 4'b1x01 === 4'b1x01, 4'b10z1 !== 4'b10z1, 4'b1001 != 4'b1000);
    $display("T|11.4.5b|%b %b", 4'b1x01 == 4'b0x01, 2'b1z == 3'b01z);
    // 11.4.6 wildcard equality
    $display("T|11.4.6a|%b %b %b", 4'b1010 ==? 4'b1x1z, 4'b1010 ==? 4'b0x1z, 4'b1x10 ==? 4'b1010);
    $display("T|11.4.6b|%b %b", 4'b1010 !=? 4'b10xx, 4'bx010 !=? 4'b1xxx);
    // 11.4.7 logical
    $display("T|11.4.7a|%b %b %b %b", 4'b0x00 && 1, 4'b0x10 && 1, 1'bx || 1, !4'b0x00);
    $display("T|11.4.7b|%b %b", 1 -> 0, 0 <-> 0);
    $display("T|11.4.7c|%b %b", 1'bx -> 1, 1'bx <-> 1'bx);
    // short-circuit
    i = 0; if (0 && (i++ > 0)) ; if (1 || (i++ > 0)) ; $display("T|11.4.7d|%0d", i);
    // 11.4.8 bitwise
    $display("T|11.4.8a|%b %b %b %b %b", 4'b01xz & 4'b1111, 4'b01xz | 4'b0000, 4'b01xz ^ 4'b1111, ~4'b01xz, 4'b01xz ~^ 4'b0101);
    $display("T|11.4.8b|%b", 4'b0000 & 4'bxxxx);
    // 11.4.9 reduction
    $display("T|11.4.9|%b %b %b %b %b %b %b", &4'b1111, &4'b1x11, |4'b0x00, |4'b0x10, ^4'b0111, ~&4'b1110, ^4'b01x0);
    // 11.4.10 shift
    s8 = -8'sd16;
    $display("T|11.4.10a|%b %b %b", s8 >>> 2, s8 >> 2, 8'b1001_0000 >>> 2);
    $display("T|11.4.10b|%b %b", 4'b1011 << 1'bx, 4'b1011 << 6);
    $display("T|11.4.10c|%b", s8 <<< 1);
    $display("T|11.4.10d|%0d", 1 << 33);
    r8 = 8'h81; $display("T|11.4.10e|%b", r8 >> -1);  // shift amount is unsigned
    // 11.4.11 conditional
    a = 4'b1010; b = 4'b0110;
    $display("T|11.4.11a|%b", 1'bx ? a : b);
    $display("T|11.4.11b|%b", 1'bx ? 4'b1111 : 4'b1111);
    $display("T|11.4.11c|%f", 1'bx ? 1.0 : 2.0);
    $display("T|11.4.11d|%0d", 1'bz ? 3 : 3);
    $display("T|11.4.11e|%b", (2'b0x) ? 2'b11 : 2'b00);
    // 11.4.12 concat/replication
    $display("T|11.4.12a|%b", {4'b1010, 2'b01});
    $display("T|11.4.12b|%b", {3{2'b10}});
    $display("T|11.4.12c|%b", {2{a, 1'b0}});
    $display("T|11.4.12d|%0d", $bits({a, {0{1'b1}}}));
    begin string s; s = {"ab", "cd"}; $display("T|11.4.12e|%s", s); s = {2{"xy"}}; $display("T|11.4.12f|%s", s); end
    r8 = {4{1'b1}}; $display("T|11.4.12g|%b", r8);
    begin int n = 3; string s; s = {n{"z"}}; $display("T|11.4.12h|%s", s); end
    // 11.4.13 set membership inside
    a = 4'd5;
    $display("T|11.4.13a|%b %b %b", a inside {1, 3, 5}, a inside {[6:9]}, a inside {[1:4], 5});
    begin int arr[3] = '{10, 11, 12}; i = 11; $display("T|11.4.13b|%b", i inside {arr}); end
    $display("T|11.4.13c|%b %b", 4'b1x00 inside {4'b1100}, 4'b1100 inside {4'b1x00});
    $display("T|11.4.13d|%b", 4'b1100 inside {4'b1?00});
    $display("T|11.4.13e|%b", 3 inside {[5:1]});
    $display("T|11.4.13f|%b", 2.5 inside {[1.0:3.0]});
    // 11.4.14 streaming
    begin
      logic [7:0] st8 = 8'b1100_0101;
      logic [15:0] st16 = 16'h1234;
      byte bq[$];
      int iq[$];
      logic [31:0] w32;
      $display("T|11.4.14a|%b", {<<{st8}});
      $display("T|11.4.14b|%h", {<<4{st16}});
      $display("T|11.4.14c|%h", {<<8{st16}});
      $display("T|11.4.14d|%h", {>>{st16}});
      $display("T|11.4.14e|%b", {<<2{st8}});
      bq = {<<8{32'h01020304}}; $display("T|11.4.14f|%p", bq);
      w32 = {>>{bq}}; $display("T|11.4.14g|%h", w32);
      {>>{st8, st16}} = 24'habcdef; $display("T|11.4.14h|%h %h", st8, st16);
      {<<{st8}} = 8'b0000_0001; $display("T|11.4.14i|%b", st8);
      iq = {>>32{64'h0000000a_0000000b}}; $display("T|11.4.14j|%p", iq);
      w32 = {<<byte{32'haabbccdd}}; $display("T|11.4.14k|%h", w32);
      begin logic [3:0] n1; logic [11:0] rest; {>>{n1, rest}} = 16'hf123; $display("T|11.4.14l|%h %h", n1, rest); end
      begin int d[]; byte b4[4] = '{1,2,3,4}; d = {>>{b4}}; $display("T|11.4.14m|%p", d); end
      begin typedef struct packed {logic [3:0] a; logic [3:0] b;} p2; p2 pp = 8'h3c; $display("T|11.4.14n|%h", {<<4{pp}}); end
      begin logic [7:0] res; res = {<<{4'b1010}}; $display("T|11.4.14o|%b", res); end
      begin byte un[$]; logic [15:0] src = 16'hbeef; un = {<< 8 {src}}; $display("T|11.4.14p|%p", un); end
    end
    // 11.5 operands: part-select, indexed
    r16 = 16'hABCD;
    $display("T|11.5.1a|%h %h %h %h", r16[15:8], r16[3 +: 8], r16[11 -: 8], r16[0 +: 4]);
    i = 14; $display("T|11.5.1b|%h", r16[i +: 4]);  // partly out of range -> x in upper
    i = -2; $display("T|11.5.1c|%h", r16[i +: 4]);
    $display("T|11.5.1d|%b", r16[16]);
    begin logic [0:15] le = 16'hABCD; $display("T|11.5.1e|%h %h %h", le[0:7], le[4 +: 4], le[15 -: 4]); end
    begin logic [3:0] xi = 4'bx; $display("T|11.5.1f|%b", r16[xi]); end
    // 11.6 expression bit lengths
    a = 4'hf; b = 4'h1;
    r8 = a + b; $display("T|11.6a|%h", r8);            // 8-bit context: 10
    $display("T|11.6b|%h", a + b);                      // 4-bit self: 0
    r8 = (a + b) >> 1; $display("T|11.6c|%h", r8);       // context 8: 08
    r8 = {a + b}; $display("T|11.6d|%h", r8);            // concat self-determined: 00
    r16 = a * b + 8'hff; $display("T|11.6e|%h", r16);
    $display("T|11.6f|%0d", $bits(a + r8));
    $display("T|11.6g|%0d %0d %0d", $bits(a == r8), $bits({a, r8}), $bits(a << 9));
    $display("T|11.6h|%0d", $bits(a ? r8 : 16'h0));
    begin logic [15:0] aa = 16'hffff, bb = 16'h1; logic [15:0] sumA; logic [16:0] sum17;
      sum17 = aa + bb; sumA = (aa + bb) >> 1; $display("T|11.6.2|%h %h", sum17, sumA);
      sumA = (aa + bb + 0) >> 1; $display("T|11.6.2b|%h", sumA); end
    // 11.7 signed expressions / 11.8 rules
    s4 = -4'sd3;
    r8 = s4; $display("T|11.8a|%b", r8);
    r8 = s4 + 4'd1; $display("T|11.8b|%b", r8);          // mixed -> unsigned, zero-extend
    r8 = s4 + 1; $display("T|11.8c|%b", r8);             // 1 is signed int -> signed
    $display("T|11.8d|%0d", s4 * 4'sd2);
    $display("T|11.8e|%0d", $signed(4'b1111) + 0);
    $display("T|11.8f|%0d", $unsigned(-4'sd1));
    $display("T|11.8g|%b", {s4} >>> 1);                  // concat is unsigned
    $display("T|11.8h|%0d", s4[3:0]);                    // part-select unsigned
    ig = -5; $display("T|11.8i|%0d %0d", ig / 2, ig >>> 1);
    $display("T|11.8j|%0d", 32'sd10 / -3);
    $display("T|11.8k|%b", 1'sb1);
    s8 = 4'sb1000; $display("T|11.8l|%0d", s8);
    s8 = 4'sb1000 + 0; $display("T|11.8m|%0d", s8);
    s8 = 4'sb1000 + 1'b0; $display("T|11.8n|%0d", s8);
    $display("T|11.8o|%0d", -3'sd1 > 3'sd2);
    // 11.8.2 real conversions
    rl = 3; rl = rl / 2; $display("T|11.8.2a|%f", rl);
    rl = 3 / 2; $display("T|11.8.2b|%f", rl);
    rl = 1.0 + 4'b1111; $display("T|11.8.2c|%f", rl);
    rl = 1.0 + 4'sb1111; $display("T|11.8.2d|%f", rl);
    i = 7.5 / 2; $display("T|11.8.2e|%0d", i);
    rl = 1'bx; $display("T|11.8.2f|%f", rl);
    rl = 2.0; $display("T|11.8.2g|%b", rl == 2);
    // 11.12 let
    $display("T|11.12|%0d %b %b", max2(3, 9), inrange(5, 1, 10), inrange(15, 1, 10));
    // 11.3.6 assignment within expression
    begin int p1, p2; if ((p1 = 4) > 3) $display("T|11.3.6|%0d", p1); p2 = (p1 += 2); $display("T|11.3.6b|%0d %0d", p1, p2); end
    // 11.4.3 power edge: x
    $display("T|11.4.3k|%b", 4'd2 ** 4'bx);
    // 11.2.2 aggregate expressions compare
    begin int u1[3] = '{1,2,3}, u2[3] = '{1,2,3}; $display("T|11.2.2|%b", u1 == u2); u2[1] = 5; $display("T|11.2.2b|%b", u1 != u2); end
    // 11.10 string comparisons in integral context
    $display("T|11.10|%b", "abc" == "abc");
    // 11.4.1 real ops: no bitwise on real; conditional mixed real
    $display("T|11.4.11f|%f", 1 ? 3 : 2.5);
  end
endmodule
