`timescale 1ns/1ps
module top;
  import "DPI-C" context function void c_entry();
  import "DPI-C" function int c_add(input int a, input int b);
  import "DPI-C" function real c_half(input real x);
  import "DPI-C" function real c_ret_real();
  import "DPI-C" function longint c_long(input longint x);
  import "DPI-C" function void c_outs(output int o, output real r);
  import "DPI-C" function byte c_byte(input byte b);
  export "DPI-C" function e_int;
  export "DPI-C" function e_long;
  export "DPI-C" function e_byte;
  export "DPI-C" function e_short;
  export "DPI-C" function e_bit;
  export "DPI-C" function e_real;
  export "DPI-C" function e_sreal;
  export "DPI-C" function e_str;
  export "DPI-C" function e_out;
  export "DPI-C" function e_uint;
  export "DPI-C" function e_half;
  export "DPI-C" function e_mhalf;
  export "DPI-C" function e_rvar;
  export "DPI-C" function sv_rep;
  export "DPI-C" function sv_repr;
  export "DPI-C" function sv_reps;
  function int e_int(); return 7.6; endfunction
  function longint e_long(); automatic time t = 100ns; return t / 1ns; endfunction
  function byte e_byte(); return -1.6; endfunction
  function shortint e_short(); return 300.5; endfunction
  function bit e_bit(); return 1.0; endfunction
  function real e_real(); return 7; endfunction
  function shortreal e_sreal(); return 2; endfunction
  function string e_str(); return "hi"; endfunction
  function void e_out(output int o, output longint ol, output real orl); o = 2.5; ol = 3.5; orl = 4; endfunction
  function int unsigned e_uint(); return -1.0; endfunction
  function int e_half(); return 2.5; endfunction
  function longint e_mhalf(); return -2.5; endfunction
  function int e_rvar(); real r; r = 9.5; return r; endfunction
  function void sv_rep(input string tag, input longint v); $display("T|%s=%0d", tag, v); endfunction
  function void sv_repr(input string tag, input real v); $display("T|%s=%f", tag, v); endfunction
  function void sv_reps(input string tag, input string v); $display("T|%s=%s", tag, v); endfunction
  int i, o; real r, rv; longint l; byte b;
  initial begin
    c_entry();
    $display("T|imp add=%0d", c_add(2.6, 3.4));
    rv = 2.5;
    $display("T|imp addv=%0d", c_add(rv, 1));
    $display("T|imp half=%f", c_half(7));
    i = c_ret_real();
    $display("T|imp retreal_to_int=%0d", i);
    $display("T|imp retreal_direct=%f", c_ret_real());
    l = c_long(1.0e10 + 0.5);
    $display("T|imp long=%0d", l);
    c_outs(o, r);
    $display("T|imp outs o=%0d r=%f", o, r);
    c_outs(r, i);
    $display("T|imp outs_swapped r=%f i=%0d", r, i);
    $display("T|imp byte=%0d", c_byte(-3.5));
  end
endmodule
