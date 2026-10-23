// Imported tasks that consume time (IEEE 1800-2017 sec. 35.5.2): the C code
// waits through exported tasks that make each kind of wait — an edge, a
// level `wait`, `fork ... join`, `wait fork`, and an output written after a
// delay; two callers at once; a call inside a loop and inside a user task;
// and C -> SV -> C recursion that waits at every level, 41 and 26 deep at
// once. Each call resumes on its own when its wait is over. Lines tagged
// `T|` are checked by the cargo test; expected values from the reference
// simulator.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_steps(input int who, input int n, input int period,
                                      output int finished_at);
  import "DPI-C" context task c_kinds(input int who, output int finished_at);
  import "DPI-C" context task c_nest(input int who, input int depth, output int levels);
  export "DPI-C" task sv_wait;
  export "DPI-C" task sv_log;
  export "DPI-C" task sv_edge;
  export "DPI-C" task sv_until;
  export "DPI-C" task sv_pair;
  export "DPI-C" task sv_spawn_wait;
  export "DPI-C" task sv_nest;
  export "DPI-C" task sv_get;

  bit clk;
  int count;
  int got = 77;
  always #5 clk = ~clk;          // posedges at 5, 15, 25, ...
  always @(posedge clk) count <= count + 1;

  task sv_wait(input int cycles); #(cycles); endtask
  task sv_log(input int who, input int step);
    $display("T|log who=%0d v=%0d t=%0t", who, step, $time);
  endtask
  task sv_edge(); @(posedge clk); endtask
  task sv_until(input int value); wait (count >= value); endtask
  task sv_pair(input int a, input int b);
    fork #(a); #(b); join
  endtask
  task sv_spawn_wait(input int d);
    fork #(d); join_none
    wait fork;
  endtask
  task automatic sv_nest(input int who, input int depth, output int levels);
    #1;
    c_nest(who, depth, levels);
  endtask
  task sv_get(output int v); #1 v = got; endtask

  // A user task that calls an imported task: inlined into the caller.
  task automatic wrap(input int who, input int n, input int period);
    int f;
    c_steps(who, n, period, f);
    $display("T|wrap who=%0d done f=%0d t=%0t", who, f, $time);
  endtask

  int f1, f2, f3, f4, lv1, lv2;
  initial begin
    fork
      c_kinds(1, f1);
      begin #3; c_kinds(2, f2); end
    join
    $display("T|kinds f1=%0d f2=%0d t=%0t", f1, f2, $time);
    fork
      for (int k = 0; k < 2; k++) c_steps(10 + k, 2, 4, f3);
      wrap(20, 3, 3);
    join
    $display("T|loop f3=%0d t=%0t", f3, $time);
    fork
      c_nest(30, 40, lv1);
      c_nest(31, 25, lv2);
    join
    $display("T|nest lv1=%0d lv2=%0d t=%0t", lv1, lv2, $time);
    $display("T|done");
    $finish;
  end
endmodule
