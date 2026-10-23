`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH
`define SVTEST_INIT \
  int failures = 0;
`define SVTEST_CHECK(expr, msg) \
  if (!(expr)) begin \
    failures++; \
    $display("FAIL @%0t : %s", $time, msg); \
  end
`define SVTEST_PASSFAIL \
  if (failures == 0) begin \
    $display("TEST_PASS"); \
  end else begin \
    $display("TEST_FAIL count=%0d", failures); \
    $fatal(1); \
  end
`endif

module nba_override (
  input  logic clk,
  input  logic rst,
  input  logic incr_sig,
  output logic active_accum_vld,
  output logic [1:0] tracking_pattern
);
  logic incr_sig_d1, incr_sig_d2, incr_sig_m1_d3;
  always @(posedge clk) begin
    incr_sig_d1    <= incr_sig;
    incr_sig_d2    <= incr_sig_d1;
    incr_sig_m1_d3 <= incr_sig_d2 - 1'b1;
    if (rst) begin
      {incr_sig_d1, incr_sig_d2, incr_sig_m1_d3} <= 'b0;
    end
  end
  always_ff @(posedge clk) begin
    if (rst) begin
      active_accum_vld <= 1'b0;
      tracking_pattern <= 2'b00;
    end else begin
      active_accum_vld <= (incr_sig_d1 ^ incr_sig_d2);
      tracking_pattern <= {incr_sig_m1_d3, incr_sig_d2};
    end
  end
endmodule
module tb_nba_override;
  `SVTEST_INIT
  logic clk;
  logic rst;
  logic incr_sig;
  logic active_accum_vld;
  logic [1:0] tracking_pattern;
  bit clk_free_running = 0;
  bit env_stabilized   = 0;
  initial begin
    clk = 1'b0;
    #3;  clk = 1'bx;
    #4;  clk = 1'b1;
    #5;  clk = 1'b0;
    #2;  clk = 1'bx;
    #6;  clk = 1'b1;
    #5;  clk_free_running = 1;
  end
  always begin
    if (clk_free_running) begin
      #5 clk = ~clk;
    end else begin
      #1;
    end
  end
  nba_override u_dut (
    .clk              (clk),
    .rst              (rst),
    .incr_sig         (incr_sig),
    .active_accum_vld (active_accum_vld),
    .tracking_pattern (tracking_pattern)
  );
  initial begin
    incr_sig = 1'b0;
    rst      = 1'bx;
    #8;  rst = 1'b0;
    #3;  rst = 1'b1;
    #4;  rst = 1'bx;
    wait(clk_free_running == 1'b1);
    @(posedge clk);
    rst = 1'b1;
    $display("  [ENV] Clock and reset stabilized. Enforcing 20+ cycle reset window.");
    repeat (22) @(posedge clk);
    env_stabilized = 1'b1;
    #1;
    `SVTEST_CHECK((u_dut.incr_sig_d1 === 1'b0),    "RESET_BUG: Internal pipeline step d1 failed reset initialization")
    `SVTEST_CHECK((u_dut.incr_sig_d2 === 1'b0),    "RESET_BUG: Internal pipeline step d2 failed reset initialization")
    `SVTEST_CHECK((u_dut.incr_sig_m1_d3 === 1'b0), "RESET_BUG: Internal pipeline step m1_d3 failed reset initialization")
    `SVTEST_CHECK((active_accum_vld === 1'b0),     "RESET_BUG: Output mask active flag failed reset assignment")
    @(posedge clk);
    #1; rst = 1'b0;
    @(posedge clk);
    #1; incr_sig = 1'b1;
    `SVTEST_CHECK((u_dut.incr_sig_d1 === 1'b0), "PROP_BUG: Spurious value forward step in cycle 1")
    @(posedge clk);
    #1; incr_sig = 1'b0;
    `SVTEST_CHECK((u_dut.incr_sig_d1 === 1'b1), "PROP_BUG: Shift tracking d1 failed to capture input")
    `SVTEST_CHECK((u_dut.incr_sig_d2 === 1'b0), "PROP_BUG: Spurious leak into tracking block d2")
    @(posedge clk);
    #1;
    `SVTEST_CHECK((u_dut.incr_sig_d1 === 1'b0),    "PROP_BUG: Shift tracking d1 failed to capture secondary input step")
    `SVTEST_CHECK((u_dut.incr_sig_d2 === 1'b1),    "PROP_BUG: Shift tracking d2 failed to pull line value from d1")
    `SVTEST_CHECK((u_dut.incr_sig_m1_d3 === 1'b1), "PROP_BUG: Mathematical transformation mapping failed on m1_d3 calculation")
    rst = 1'b1;
    repeat(2) @(posedge clk);
    #1;
    `SVTEST_CHECK((u_dut.incr_sig_d1 === 1'b0), "NBA_BUG: Reset failed to claim priority over pipeline data in step d1")
    `SVTEST_CHECK((u_dut.incr_sig_d2 === 1'b0), "NBA_BUG: Reset failed to claim priority over pipeline data in step d2")
    `SVTEST_CHECK((u_dut.incr_sig_m1_d3 === 1'b0), "NBA_BUG: Reset failed to claim priority over pipeline data in step m1_d3")
    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
