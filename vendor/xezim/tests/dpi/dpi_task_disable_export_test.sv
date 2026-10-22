// `disable` of an EXPORTED task while the C code of an imported task waits
// in it (IEEE 1800-2017 sec. 35.9): the export returns 0 at once, the
// imported task is not disabled, and the C code carries on. Lines tagged
// `T|` are checked by the cargo test; expected values from the reference
// simulator.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_guarded(input int who, input int n, input int period);
  import "DPI-C" function int c_seen(input int who);
  export "DPI-C" task sv_wait;
  export "DPI-C" task sv_log;
  task sv_wait(input int cycles); #(cycles); endtask
  task sv_log(input int who, input int step);
    $display("T|log who=%0d step=%0d t=%0t", who, step, $time);
  endtask
  initial begin
    c_guarded(7, 3, 10);
    $display("T|returned seen=%0d t=%0t", c_seen(7), $time);
    $display("T|done");
  end
  initial #15 disable sv_wait;
endmodule
