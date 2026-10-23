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

module credit_pool #(
  parameter int CDTS_INIT = 288
)(
  input  logic        clk,
  input  logic        rst_l,
  input  logic [1:0]  consume_credit_vld,
  input  logic [1:0]  return_credit_vld,
  output logic [1:0][9:0] cdts
);
  logic [9:0] cdts_default;
  assign cdts_default = CDTS_INIT >> 3;
  always_ff @(posedge clk) begin
    if (!rst_l) begin
      cdts <= '{default: cdts_default};
    end else begin
      for (int i = 0; i < 2; i++) begin
        if (consume_credit_vld[i] && !return_credit_vld[i] && (cdts[i] > 0)) begin
          cdts[i] <= cdts[i] - 1'b1;
        end else if (return_credit_vld[i] && !consume_credit_vld[i] && (cdts[i] < 10'h3FF)) begin
          cdts[i] <= cdts[i] + 1'b1;
        end
      end
    end
  end
endmodule
module tb_credit_pool;
  `SVTEST_INIT
  logic        clk;
  logic        rst_l;
  logic [1:0]  consume_credit_vld;
  logic [1:0]  return_credit_vld;
  logic [1:0][9:0] cdts;
  initial clk = 0;
  always #5 clk = ~clk;
  credit_pool #(
    .CDTS_INIT(288)
  ) u_dut (
    .clk                (clk),
    .rst_l              (rst_l),
    .consume_credit_vld (consume_credit_vld),
    .return_credit_vld  (return_credit_vld),
    .cdts               (cdts)
  );
  class credit_stim_c;
    bit shared_mode_bit;
    rand bit [1:0] rand_consume;
    rand bit [1:0] rand_return;
    constraint c_mode_distribution {
      if (shared_mode_bit == 1'b1) {
        rand_consume == rand_return;
      } else {
        rand_consume == 2'b11;
        rand_return  == 2'b00;
      }
    }
  endclass
  initial begin
    credit_stim_c stim = new();
    rst_l              = 1'b1;
    consume_credit_vld = 2'b00;
    return_credit_vld  = 2'b00;
    stim.shared_mode_bit = 1'b1 ;
    #1; rst_l = 1'b0;
    #20;
    `SVTEST_CHECK((u_dut.cdts_default == 36), "PARAM_BUG: Shift alignment logic mismatch on default value calculation!")
    `SVTEST_CHECK((cdts[0] == 36), "RESET_BUG: Lane 0 default array parameter reflection layout failed.")
    `SVTEST_CHECK((cdts[1] == 36), "RESET_BUG: Lane 1 default array parameter reflection layout failed.")
    @(posedge clk);
    #1; rst_l = 1'b1;
    @(posedge clk);
    $display("Executing Transaction Checks: Safe Mode Active");
    stim.shared_mode_bit = 1'b1;
    repeat (10) begin
      `SVTEST_CHECK(stim.randomize(), "RAND_ERROR: Stimulus package validation engine fault.")
      consume_credit_vld = stim.rand_consume;
      return_credit_vld  = stim.rand_return;
      @(posedge clk);
      #1;
      `SVTEST_CHECK((cdts[0] == 36), "STRUCT_BUG: System credit allocation variance observed under Safe Mode.")
      `SVTEST_CHECK((cdts[1] == 36), "STRUCT_BUG: System credit allocation variance observed under Safe Mode.")
    end
    $display("Executing Transaction Checks: Stress Mode Active");
    stim.shared_mode_bit = 1'b0;
    repeat (40) begin
      `SVTEST_CHECK(stim.randomize(), "RAND_ERROR: Stimulus package validation engine fault.")
      consume_credit_vld = stim.rand_consume;
      return_credit_vld  = stim.rand_return;
      @(posedge clk);
      #1;
    end
    `SVTEST_CHECK((cdts[0] == 0), "FLOOR_BUG: Structural credit track underflow guard failure on Lane 0.")
    `SVTEST_CHECK((cdts[1] == 0), "FLOOR_BUG: Structural credit track underflow guard failure on Lane 1.")
    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
