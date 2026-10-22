// top: c20
`timescale 1ns/10ps
module c20sub; initial begin #3.333; $display("T|20.3sub|%0t %0d %f", $time, $stime, $realtime); end endmodule
`timescale 1us/1ns
module c20;
  c20sub u();
  logic [7:0] v;
  real r;
  int i;
  initial begin
    // 20.3 time functions
    #1.2345;
    $display("T|20.3a|%0t %0d %0d %f", $time, $time, $stime, $realtime);
    $display("T|20.3b|%t|", $time);
    // 20.4 $timeformat
    $timeformat(-9, 2, " ns", 12);
    $display("T|20.4.2a|[%t]", $realtime);
    $display("T|20.4.2b|[%0t]", $realtime);
    $timeformat(-6, 0, "us", 0);
    $display("T|20.4.2c|[%t]", $time);
    $timeformat(-12, 3, "ps", 15);
    $display("T|20.4.2d|[%t]", 1.5);
    $timeformat;
    $display("T|20.4.2e|[%t]", $realtime);
    $printtimescale;
    $printtimescale(c20.u);
    // 20.5 conversions
    $display("T|20.5a|%f %0d %0d", $itor(5), $rtoi(-3.9), $rtoi(3.9));
    $display("T|20.5b|%h", $realtobits(1.0));
    $display("T|20.5c|%f", $bitstoreal(64'h4000000000000000));
    $display("T|20.5d|%h %f", $shortrealtobits(shortreal'(1.5)), $bitstoshortreal(32'h40490fdb));
    $display("T|20.5e|%0d %0d", $signed(4'b1100), $unsigned(4'sb1100));
    // 20.6 data query
    $display("T|20.6.2a|%0d %0d", $bits(v), $bits(struct packed {logic a; logic [2:0] b;})); $display("T|20.6.2r|%0d", $bits(real));
    begin int dq[$] = {1,2,3}; $display("T|20.6.2b|%0d", $bits(dq)); end
    $display("T|20.6.1|%s %s %s", $typename(v), $typename(int), $typename(r));
    begin typedef enum {X1, X2} e_t; e_t ev; $display("T|20.6.1b|%s", $typename(ev)); end
    $display("T|20.6.3|%0d %0d", $isunbounded(5), 0);
    // 20.7 array query
    begin logic [7:0][3:0] pk [2:5][1:0]; int dd[]; int aq[$] = {1,2,3}; int ac[string];
      dd = new[4]; ac["a"] = 1;
      $display("T|20.7a|%0d %0d %0d", $dimensions(pk), $unpacked_dimensions(pk), $dimensions(int));
      $display("T|20.7b|%0d %0d %0d %0d %0d %0d", $left(pk), $right(pk), $low(pk), $high(pk), $increment(pk), $size(pk));
      $display("T|20.7c|%0d %0d %0d %0d", $left(pk, 3), $right(pk, 4), $size(pk, 3), $increment(pk, 2));
      $display("T|20.7d|%0d %0d %0d %0d", $size(dd), $right(dd), $high(aq), $size(ac));
      $display("T|20.7e|%0d %0d", $left(v), $increment(v));
      $display("T|20.7f|%0d", $size(pk, 5));
    end
    // 20.8 math
    $display("T|20.8.1a|%0d %0d %0d %0d %0d", $clog2(0), $clog2(1), $clog2(2), $clog2(5), $clog2(64'h100000000));
    $display("T|20.8.2a|%f %f %f %f", $ln(1.0), $log10(1000.0), $exp(0.0), $sqrt(16.0));
    $display("T|20.8.2b|%f %f %f %f", $pow(2.0, 3.0), $floor(-1.5), $ceil(-1.5), $sin(0.0));
    $display("T|20.8.2c|%f %f %f %f", $cos(0.0), $tan(0.0), $asin(1.0), $acos(1.0));
    $display("T|20.8.2d|%f %f %f %f", $atan(1.0), $atan2(1.0, 1.0), $hypot(3.0, 4.0), $sinh(0.0));
    $display("T|20.8.2e|%f %f %f %f %f", $cosh(0.0), $tanh(0.0), $asinh(0.0), $acosh(1.0), $atanh(0.0));
    $display("T|20.8.2f|%f", $sqrt(-1.0));
    // 20.9 bit vector system functions
    $display("T|20.9a|%0d %0d %0d", $countbits(8'b1010_xz01, '1), $countbits(8'b1010_xz01, 'x, 'z), $countbits(8'b1010_xz01, '0));
    $display("T|20.9b|%0d %0d", $countones(8'b1011_x001), $countones(0));
    $display("T|20.9c|%b %b %b", $onehot(8'b0001_0000), $onehot(8'b0001_1000), $onehot(0));
    $display("T|20.9d|%b %b %b", $onehot0(0), $onehot0(8'b0100_0000), $onehot0(3));
    $display("T|20.9e|%b %b %b", $isunknown(4'b10z1), $isunknown(4'b1001), $isunknown(4'bx));
    // 20.15 random
    begin int seed = 5; int r1, r2, r3; int s2 = 5;
      r1 = $random(seed); r2 = $random(seed); r3 = $random(s2);
      $display("T|20.15.1a|%0d %0d %0d same=%0d", r1, r2, r3, r1 == r3);
      seed = 1; $display("T|20.15.2a|%0d %0d %0d", $dist_uniform(seed, 0, 100), $dist_uniform(seed, 0, 100), seed);
      seed = 7; $display("T|20.15.2b|%0d %0d", $dist_normal(seed, 50, 10), $dist_exponential(seed, 10));
      seed = 7; $display("T|20.15.2c|%0d %0d %0d", $dist_poisson(seed, 5), $dist_chi_square(seed, 3), $dist_t(seed, 4));
      seed = 7; $display("T|20.15.2d|%0d", $dist_erlang(seed, 2, 10));
      seed = 3; $display("T|20.15.1b|%0d", $random(seed) % 10);
    end
    begin int unsigned u1, u2; u1 = $urandom(42); u2 = $urandom(42); $display("T|20.15.3a|same=%0d", u1 == u2);
      repeat (20) begin u1 = $urandom_range(3, 7); if (u1 < 3 || u1 > 7) $display("T|20.15.3b|bad %0d", u1); end
      repeat (20) begin u1 = $urandom_range(7, 3); if (u1 < 3 || u1 > 7) $display("T|20.15.3c|bad %0d", u1); end
      repeat (20) begin u1 = $urandom_range(2); if (u1 > 2) $display("T|20.15.3d|bad %0d", u1); end
      $display("T|20.15.3|range ok"); end
    // 20.10 severity tasks
    $info("info msg %0d", 1);
    $warning("warn msg");
    $error("err msg");
    $display("T|20.10|after error");
    // 20.2 $stop / $finish level
    #1 $display("T|20.2|before finish t=%0t", $time);
    $finish(2);
    $display("T|20.2b|after finish (should not print)");
  end
  final $display("T|20.2c|final");
endmodule
