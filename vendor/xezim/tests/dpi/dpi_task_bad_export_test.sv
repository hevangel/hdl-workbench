// A disabled imported task that calls an export breaks the DPI disable
// protocol (IEEE 1800-2017 sec. 35.9 d): a fatal error, as in the reference
// simulator.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_bad_export(input int period);
  export "DPI-C" task sv_wait;
  export "DPI-C" task sv_log;
  task sv_wait(input int cycles); #(cycles); endtask
  task sv_log(input int who, input int step);
    $display("T|log who=%0d step=%0d t=%0t", who, step, $time);
  endtask
  initial begin
    fork : blk c_bad_export(10); join_none
    #5 disable blk;
    #1 $display("T|after t=%0t", $time);
    $display("T|done");
  end
endmodule
