// DPI imported tasks (IEEE 1800-2017 sec. 35.5.2, 35.6.1): time consumed
// through an exported task, outputs written when the task returns, a task
// called again after it returned, and a zero-time task.
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
    c_instant(21, d);
    $display("T|instant d=%0d t=%0t", d, $time);

    // one caller: 3 waits of 5
    c_worker(0, 3, 5, s0, e0);
    $display("T|solo started=%0d elapsed=%0d t=%0t", s0, e0, $time);

    // the same task again, after the first call returned
    c_worker(1, 2, 7, s1, e1);
    $display("T|again started=%0d elapsed=%0d t=%0t", s1, e1, $time);

    $display("T|done");
  end
endmodule
