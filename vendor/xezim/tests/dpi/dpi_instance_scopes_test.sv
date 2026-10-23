// Exports from instantiated modules (IEEE 1800-2017 sec. 35.5.3): three
// instances of `unit` (one nested) export the same subroutines; a context
// import called through an instance, or from inside it, reaches that
// instance's copy (its parameter, its `%m`); svSetScope selects one by path;
// an imported task in each instance waits through its own exported task;
// and an export of top is found from an instance's scope. Lines tagged `T|`
// are checked by the cargo test; expected values from the reference
// simulator.
`timescale 1ns/1ns
module unit #(parameter int ID = 0) ();
  import "DPI-C" context function int c_ask_id();
  import "DPI-C" context task c_tick(input int n);
  import "DPI-C" context function int c_call_top();
  export "DPI-C" function sv_get_id;
  export "DPI-C" task sv_tick;
  export "DPI-C" function sv_where;
  function int sv_get_id(); return ID; endfunction
  task sv_tick(input int n); #(n); endtask
  function void sv_where(); $display("T|where %m t=%0t", $time); endfunction
  initial begin
    #1 c_tick(ID);
    $display("T|ticked %m id=%0d t=%0t", ID, $time);
  end
  initial #12 $display("T|up %m=%0d", c_call_top());
endmodule

module wrapper ();
  unit #(.ID(7)) u ();
endmodule

module top;
  import "DPI-C" context function int c_ask_in(input string path);
  export "DPI-C" function sv_top_val;
  function int sv_top_val(); return 42; endfunction
  unit #(.ID(3)) ua ();
  unit #(.ID(5)) ub ();
  wrapper w ();
  initial begin
    $display("T|ids a=%0d b=%0d w=%0d", ua.c_ask_id(), ub.c_ask_id(), w.u.c_ask_id());
    $display("T|set a=%0d b=%0d w=%0d", c_ask_in("top.ua"), c_ask_in("top.ub"),
             c_ask_in("top.w.u"));
    #20 $display("T|done");
  end
endmodule
