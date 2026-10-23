// A function exported from an INSTANTIATED module (IEEE 1800-2017
// sec. 35.5.3): a context import calls the export of the instance it was
// called through. Shares dpi_scope_exports.c.
module unit #(parameter int ID = 0) ();
  import "DPI-C" context function int c_ask_id();
  export "DPI-C" function sv_get_id;
  function int sv_get_id();
    return ID;
  endfunction
endmodule

module top;
  export "DPI-C" function sv_fact;
  import "DPI-C" context function int c_fact(input int n);
  function int sv_fact(input int k);
    return c_fact(k);
  endfunction
  unit #(.ID(11)) ua ();
  unit #(.ID(22)) ub ();
  initial begin
    $display("T|ids a=%0d b=%0d", ua.c_ask_id(), ub.c_ask_id());
    $display("T|done");
  end
endmodule
