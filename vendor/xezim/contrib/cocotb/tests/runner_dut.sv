// Design for test_xezim_runner.py. Each output shows whether an option the
// runner was given reached the run. No `timescale: the runner's applies.
`include "runner_inc.svh"

module runner_dut #(
  parameter int W = 1,
  parameter [5:0] P = 6'd0
) (
  output int w,
  output logic [5:0] p,
  output logic foo,
  output logic bar,
  output logic [15:0] msg,
  output int num,
  output int inc,
  output int pa,
  output logic fine
);
  assign w = W;
  assign p = P;
`ifdef FOO
  assign foo = 1'b1;
`else
  assign foo = 1'b0;
`endif
`ifdef BAR
  assign bar = 1'b1;
`else
  assign bar = 1'b0;
`endif
`ifdef MSG
  assign msg = `MSG;
`else
  assign msg = '0;
`endif
`ifdef NUM
  assign num = `NUM;
`else
  assign num = 0;
`endif
  assign inc = `INC_VAL;

  int pa_v = 0;
  initial void'($value$plusargs("PA=%d", pa_v));
  assign pa = pa_v;

  // 1 ps at 1ns/1ps; at the 1ns/1ns default the delay rounds to 0 (3.14.2.2).
  realtime t = 0;
  initial #0.001 t = $realtime;
  assign fine = t > 0;
endmodule

// Not the toplevel: elaborated only if the runner fails to select runner_dut.
module runner_other;
  initial $fatal(1, "runner_other is not the toplevel");
endmodule
