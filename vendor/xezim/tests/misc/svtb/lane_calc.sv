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

module lane_calc (
  input  logic       clk,
  input  logic       rst_l,
  input  logic [2:0] cfg_log_burst,
  output logic [3:0] active_lane_mask,
  output logic       lane_overflow_err
);
  logic [4:0] lanes_per_grp;
  always_ff @(posedge clk) begin
    if (!rst_l) begin
      lanes_per_grp     <= '0;
      active_lane_mask   <= '0;
      lane_overflow_err  <= 1'b0;
    end else begin
      lanes_per_grp     <= (1 << cfg_log_burst) >> 3;
      if (lanes_per_grp == 5'd0) begin
        active_lane_mask  <= 4'b0001;
        lane_overflow_err <= 1'b0;
      end else if (lanes_per_grp == 5'd1) begin
        active_lane_mask  <= 4'b0011;
        lane_overflow_err <= 1'b0;
      end else if (lanes_per_grp == 5'd2) begin
        active_lane_mask  <= 4'b1111;
        lane_overflow_err <= 1'b0;
      end else begin
        active_lane_mask  <= 4'b0000;
        lane_overflow_err <= 1'b1;
      end
    end
  end
endmodule
module tb_lane_calc;
  `SVTEST_INIT
  logic       clk;
  logic       rst_l;
  logic [2:0] cfg_log_burst;
  logic [3:0] active_lane_mask;
  logic       lane_overflow_err;
  initial clk = 0;
  always #5 clk = ~clk;
  lane_calc u_dut (
    .clk                  (clk),
    .rst_l                (rst_l),
    .cfg_log_burst  (cfg_log_burst),
    .active_lane_mask     (active_lane_mask),
    .lane_overflow_err    (lane_overflow_err)
  );
  initial begin
    rst_l               = 1'b1;
    cfg_log_burst = 3'b000;
    #1; rst_l = 1'b0;
    #20; rst_l = 1'b1;
    @(posedge clk);
    cfg_log_burst = 3'd3;
    repeat(2) @(posedge clk); #1;
    `SVTEST_CHECK((u_dut.lanes_per_grp == 5'd1), "CALC_BUG: Bit shift calculation failure for scale 3")
    `SVTEST_CHECK((active_lane_mask == 4'b0011),  "MASK_BUG: Lane mask layout mismatch for single lane mode")
    `SVTEST_CHECK((lane_overflow_err == 1'b0),    "FLAG_BUG: Spurious overflow exception raised for scale 3")
    cfg_log_burst = 3'd2;
    repeat(2) @(posedge clk); #1;
    `SVTEST_CHECK((u_dut.lanes_per_grp == 5'd0), "CALC_BUG: Bit shift floor boundary violation for scale 2")
    `SVTEST_CHECK((active_lane_mask == 4'b0001),  "MASK_BUG: Base layout mask mapping fault for floor boundary")
    `SVTEST_CHECK((lane_overflow_err == 1'b0),    "FLAG_BUG: Overflow flag fault for scale 2")
    cfg_log_burst = 3'd4;
    repeat(2) @(posedge clk); #1;
    `SVTEST_CHECK((u_dut.lanes_per_grp == 5'd2), "CALC_BUG: Shift calculation failure for max active threshold 4")
    `SVTEST_CHECK((active_lane_mask == 4'b1111),  "MASK_BUG: Multi-lane mask layout binding mismatch for scale 4")
    `SVTEST_CHECK((lane_overflow_err == 1'b0),    "FLAG_BUG: Unintended boundary flag validation trip for scale 4")
    cfg_log_burst = 3'd5;
    repeat(2) @(posedge clk); #1;
    `SVTEST_CHECK((u_dut.lanes_per_grp == 5'd4), "CALC_BUG: Sizing track overflow tracking evaluation mismatch")
    `SVTEST_CHECK((active_lane_mask == 4'b0000),  "MASK_BUG: Safety isolation circuit failed to dump output lane bits")
    `SVTEST_CHECK((lane_overflow_err == 1'b1),    "FLAG_BUG: Safety critical calculation error tracking omitted flag raise")
    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
