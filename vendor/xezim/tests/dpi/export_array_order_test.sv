// #291: unpacked-array formals of an exported function, passed from C by
// pointer (IEEE 1800 §35.5.6, Annex H.7.5). C element 0 is the lowest index
// of each dimension, whichever way the range runs; the first dimension is
// outermost. Expected values from the reference simulator.
module top;
  import "DPI-C" context function void c_main();
  function void put(string s); $display("T|%s", s); endfunction
  export "DPI-C" function put;
  function void f(input int a[5:2], input int b[2:5], input int c[1:0][0:2], output int o[3:1]);
    $display("T|a[5]=%0d a[2]=%0d b[2]=%0d b[5]=%0d", a[5], a[2], b[2], b[5]);
    $display("T|c[1][0]=%0d c[1][2]=%0d c[0][0]=%0d c[0][2]=%0d", c[1][0], c[1][2], c[0][0], c[0][2]);
    o[3] = 33; o[1] = 11; o[2] = 22;
  endfunction
  export "DPI-C" function f;
  initial c_main();
endmodule
