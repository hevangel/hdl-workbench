// Imported tasks called through instance paths from two processes at once
// (IEEE 1800-2017 sec. 35.5.2, 35.5.3): each call runs in its instance's
// scope, waits through that instance's exported task and resumes on its
// own. Shares dpi_instance_scopes.c. Lines tagged `T|` are checked by the
// cargo test; expected values from the reference simulator.
`timescale 1ns/1ns
module unit #(parameter int ID = 0) ();
  import "DPI-C" context task c_tick(input int n);
  export "DPI-C" task sv_tick;
  export "DPI-C" function sv_where;
  task sv_tick(input int n); #(n); endtask
  function void sv_where(); $display("T|where %m t=%0t", $time); endfunction
endmodule
module top;
  unit #(.ID(3)) ua ();
  unit #(.ID(5)) ub ();
  initial begin
    fork
      ua.c_tick(4);
      ub.c_tick(2);
    join
    $display("T|done t=%0t", $time);
  end
endmodule
