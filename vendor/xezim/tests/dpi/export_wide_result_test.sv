// #291: a DPI function result is a small value (IEEE 1800 §35.5.5): an
// exported function returning a packed vector wider than 64 bits is not a
// legal export, and the run stops before it starts, as in the reference
// simulator.
module top;
  import "DPI-C" context function void c_main();
  function logic [127:0] wide(input logic [127:0] v); return v; endfunction
  export "DPI-C" function wide;
  initial c_main();
endmodule
