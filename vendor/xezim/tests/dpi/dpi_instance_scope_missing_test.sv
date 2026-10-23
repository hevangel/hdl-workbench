// An export declared only in an instance is not visible from top's scope
// (IEEE 1800-2017 sec. 35.5.3): calling it from a context import in top is
// a fatal error, as in the reference simulator.
module unit #(parameter int ID = 0) ();
  export "DPI-C" function sv_get_id;
  function int sv_get_id(); return ID; endfunction
endmodule

module top;
  import "DPI-C" context function int c_ask_id();
  unit #(.ID(3)) ua ();
  initial begin
    $display("T|from-top=%0d", c_ask_id());
    $display("T|done");
  end
endmodule
