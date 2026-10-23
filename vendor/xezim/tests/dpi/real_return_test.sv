// #324: exported DPI functions with integral return types must VALUE-convert
// a real-valued return expression (rounding to nearest, ties away from zero,
// §5.2/§6.12), not bit-reinterpret its IEEE-754 pattern. Each returns a real
// expression; the C side (real_return.c) reads them as long long / int / short.
module top;
  import "DPI-C" context function void c_entry();
  import "DPI-C" context function int c_take_int(input int v);
  import "DPI-C" context function void c_take_time(input time v);
  import "DPI-C" context function void c_take_integer(input integer v);
  import "DPI-C" context function void c_take_bit(input bit [3:0] v);
  export "DPI-C" function get_drain_time;
  export "DPI-C" function get_i32;
  export "DPI-C" function get_i16;
  export "DPI-C" function sv_report;

  function longint get_drain_time();
    automatic time t = 100ns;
    return t / 1ns;                // real 100.0 -> long long 100
  endfunction

  function int get_i32();
    return 3.75;                   // real 3.75 -> int 4 (round half away)
  endfunction

  function shortint get_i16();
    return -3.5;                   // real -3.5 -> short -4
  endfunction

  // Called by C to confirm the exported value is 100 (value-converted).
  function void sv_report(input longint v);
    if (v == 100)
      $display("PASS: get_drain_time returned %0d", v);
    else
      $display("FAIL: get_drain_time returned %0d (0x%016h) expected 100", v, v);
  endfunction

  initial begin
    // #324 argument side: a real actual into an int formal round-converts.
    if (c_take_int(3.75) == 4)
      $display("PASS: c_take_int(3.75) == 4");
    else
      $display("FAIL: c_take_int(3.75) == %0d, expected 4", c_take_int(3.75));
    // #324 cousin: real actuals into packed time/integer/bit vector formals
    // value-convert (round), not bit-reinterpret. The C side prints T|T4, T|I4.
    c_take_time(3.75);   // time   -> 4
    c_take_integer(3.75);// integer -> 4
    c_take_bit(3.75);    // bit [3:0] -> 4
    c_entry();
  end
endmodule