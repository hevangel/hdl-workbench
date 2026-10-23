// The DPI disable protocol (IEEE 1800-2017 sec. 35.9) for a process
// waiting inside an imported task when it is killed: by `disable` of its
// named fork block, by process::kill(), and by `disable fork`. The waiting
// exported task returns 1, svIsDisabledState() is 1 (seen = 11), and the C
// code acknowledges and returns 1. Lines tagged `T|` are checked by the
// cargo test; expected values from the reference simulator.
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
  process p;
  initial begin
    // disable of a named fork block
    fork : blk
      c_guarded(1, 5, 10);
    join_none
    #25 disable blk;
    #1 $display("T|fork-block seen=%0d t=%0t", c_seen(1), $time);
    // process::kill
    fork begin p = process::self(); c_guarded(2, 5, 10); end join_none
    #25 p.kill();
    #1 $display("T|kill seen=%0d t=%0t", c_seen(2), $time);
    // disable fork
    fork c_guarded(3, 5, 10); join_none
    #25 disable fork;
    #1 $display("T|disable-fork seen=%0d t=%0t", c_seen(3), $time);
    // a call that is not disabled finishes normally
    c_guarded(4, 2, 3);
    $display("T|normal seen=%0d t=%0t", c_seen(4), $time);
    $display("T|done");
  end
endmodule
