// A disabled imported task that returns 0 breaks the DPI disable protocol
// (IEEE 1800-2017 sec. 35.9 b): a fatal error, as in the reference
// simulator.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_bad_return(input int period);
  export "DPI-C" task sv_wait;
  task sv_wait(input int cycles); #(cycles); endtask
  initial begin
    fork : blk c_bad_return(10); join_none
    #5 disable blk;
    #1 $display("T|after t=%0t", $time);
    $display("T|done");
  end
endmodule
