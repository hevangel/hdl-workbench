// Self-checking testcase for xezim DPI return-value bug:
// an exported `longint` function returning `time / 1ns` should yield 100,
// not the bit-pattern of real 100.0.
module top;
  import "DPI-C" context function void c_entry();
  export "DPI-C" function get_drain_time;
  export "DPI-C" function sv_report;

  function longint get_drain_time();
    automatic time t = 100ns;
    return t / 1ns;
  endfunction

  function void sv_report(input longint v);
    if (v == 100)
      $display("PASS: get_drain_time returned %0d", v);
    else
      $display("FAIL: get_drain_time returned %0d (0x%016h) expected 100", v, v);
  endfunction

  initial begin
    c_entry();
  end
endmodule
