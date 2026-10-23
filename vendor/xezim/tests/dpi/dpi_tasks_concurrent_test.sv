// Two processes inside the same DPI imported task at once (IEEE 1800-2017
// sec. 35.5.2): each consumes time through an exported task with its own
// period, so their steps interleave and each returns when its own waits are
// done. Shares dpi_tasks.c with dpi_tasks_test.sv.
// Lines tagged `T|` are checked by the cargo test.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_worker(input int who, input int n, input int period,
                                       output int started, output int elapsed);
  import "DPI-C" task c_instant(input int x, output int doubled);
  export "DPI-C" task sv_wait;
  export "DPI-C" task sv_log;

  task sv_wait(input int cycles);
    #(cycles);
  endtask

  task sv_log(input int who, input int step);
    $display("T|step who=%0d step=%0d t=%0t", who, step, $time);
  endtask

  int s0, e0, s1, e1, s2, e2, d;

  initial begin
    // two callers in the same task at once, with different periods
    fork
      c_worker(1, 2, 7, s1, e1);
      c_worker(2, 3, 3, s2, e2);
    join
    $display("T|pair w1 started=%0d elapsed=%0d w2 started=%0d elapsed=%0d t=%0t",
             s1, e1, s2, e2, $time);
    $display("T|done");
  end
endmodule
