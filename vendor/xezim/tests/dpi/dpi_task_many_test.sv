// Two hundred processes inside the same imported task at once, each
// waiting through an exported task on its own period (IEEE 1800-2017 sec.
// 35.5.2). Lines tagged `T|` are checked by the cargo test.
`timescale 1ns/1ns
module top;
  import "DPI-C" context task c_steps(input int who, input int n, input int period,
                                      output int finished_at);
  export "DPI-C" task sv_wait;
  export "DPI-C" task sv_log;
  task sv_wait(input int cycles); #(cycles); endtask
  task sv_log(input int who, input int step); endtask
  int fin[200];
  initial begin
    for (int k = 0; k < 200; k++) begin
      automatic int kk = k;
      fork c_steps(kk, 3, 1 + kk % 7, fin[kk]); join_none
    end
    wait fork;
    begin
      automatic int sum = 0, mx = 0;
      foreach (fin[k]) begin sum += fin[k]; if (fin[k] > mx) mx = fin[k]; end
      $display("T|many sum=%0d max=%0d t=%0t", sum, mx, $time);
    end
    $display("T|done");
  end
endmodule
