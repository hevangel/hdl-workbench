// The DPI disable protocol (IEEE 1800-2017 sec. 35.9) when the waiting
// exported task's own code kills the calling process: the export returns 1,
// the C code acknowledges and returns 1, and the process does not carry on.
// Lines tagged `T|` are checked by the cargo test; expected values from the
// reference simulator.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_guarded(input int who, input int n, input int period);
  import "DPI-C" function int c_seen(input int who);
  export "DPI-C" task sv_wait;
  export "DPI-C" task sv_log;
  task sv_wait(input int cycles);
    #(cycles);
    if ($time >= 20) begin
      automatic process me = process::self();
      $display("T|self-kill t=%0t", $time);
      me.kill();
      $display("T|after kill (must not print)");
    end
  endtask
  task sv_log(input int who, input int step);
    $display("T|log who=%0d step=%0d t=%0t", who, step, $time);
  endtask
  initial begin
    fork begin c_guarded(6, 5, 10); $display("T|rest ran (must not print)"); end join_none
    #30 $display("T|seen=%0d t=%0t", c_seen(6), $time);
    $display("T|done");
  end
endmodule
