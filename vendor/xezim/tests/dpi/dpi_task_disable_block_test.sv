// The DPI disable protocol (IEEE 1800-2017 sec. 35.9, 9.6.2) when another
// process disables a named block around the call: the imported task unwinds
// (seen = 11), the rest of the block is skipped and the process carries on
// after it. Lines tagged `T|` are checked by the cargo test.
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
  initial begin : runner
    begin : inner
      c_guarded(5, 5, 10);
      $display("T|rest of inner ran t=%0t", $time);
    end
    $display("T|after inner seen=%0d t=%0t", c_seen(5), $time);
    $display("T|done");
  end
  initial #35 disable runner.inner;
endmodule
