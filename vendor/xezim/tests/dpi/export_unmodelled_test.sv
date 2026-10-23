// #291: an exported function with a formal DPI cannot pass (an `event`)
// is not callable from C. The startup message says so, and each call from
// C reports an error at the call site and returns 0 without running the
// function; the other exports keep working.
module top;
  import "DPI-C" context function void c_main();
  int hits = 0;
  function void put(string s); $display("T|%s", s); endfunction
  export "DPI-C" function put;
  function int f_ev(input int a, input event e); hits++; return a; endfunction
  export "DPI-C" function f_ev;
  function int ok(input int a); hits++; return a + 1; endfunction
  export "DPI-C" function ok;
  initial begin
    c_main();
    $display("T|hits=%0d", hits);
    $finish;
  end
endmodule
