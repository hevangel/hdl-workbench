// $finish while two processes wait inside imported tasks (IEEE 1800-2017
// sec. 35.5.2, 20.2): the run ends at once, the suspended calls are left.
// Lines tagged `T|` are checked by the cargo test.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_steps(input int who, input int n, input int period,
                                      output int finished_at);
  export "DPI-C" task sv_wait;
  export "DPI-C" task sv_log;
  task sv_wait(input int cycles); #(cycles); endtask
  task sv_log(input int who, input int step);
    $display("T|log who=%0d step=%0d t=%0t", who, step, $time);
  endtask
  int f;
  initial begin
    fork c_steps(1, 100, 10, f); c_steps(2, 100, 7, f); join_none
    #25 $display("T|finish t=%0t", $time);
    $finish;
  end
endmodule
