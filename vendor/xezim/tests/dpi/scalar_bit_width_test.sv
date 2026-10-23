// Regression: scalar `bit` DPI formals carry bit 0 only; real->longint wraps
// at the 2^63 boundary. C prints what it received; the .rs harness asserts
// the bytes match the reference simulator.
module top;
  import "DPI-C" context function void bitw_take(input bit a, input bit b, input bit c, input bit d);
  import "DPI-C" context function void bitw_take_long(input longint v);
  import "DPI-C" context function void bitw_toggle(inout bit b);
  import "DPI-C" context function void bitw_set(output bit b);
  bit io;

  initial begin
    bitw_take(2, 1.75, -0.5, 300);   // reduced to bit 0: 0, 0, 1, 0
    bitw_take_long(9223372036854775808.0); // 2^63 -> INT64_MIN
    bitw_take_long(18446744073709551616.0); // 2^64 -> 0
    bitw_toggle(io);                  // seed 0 -> C toggles -> 1
    $display("T|IO %0d", io);
    bitw_set(io);
    $display("T|OUT %0d", io);
    $finish;
  end
endmodule