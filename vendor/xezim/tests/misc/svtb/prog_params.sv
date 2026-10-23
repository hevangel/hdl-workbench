package test_tracker_pkg;
  int failures = 0;
  bit [7:0] test_status = '1;
  bit       top_done = 1'b0;
endpackage
`define SVTEST_INIT2 import test_tracker_pkg::*;
`define SVTEST_CHECK_TRACK(index, expr, msg) \
  if (!(expr)) begin \
    test_tracker_pkg::failures++; \
    test_tracker_pkg::test_status[index] = 1'b0; \
    $display("FAIL: %s at time %0t ps", msg, $time); \
  end
`define SVTEST_PASSFAIL2 \
  if (test_tracker_pkg::failures == 0) begin \
    $display("TEST_PASS"); \
  end else begin \
    $display("TEST_FAIL count=%0d", test_tracker_pkg::failures); \
    $fatal(1); \
  end
package infra_pkg;
  typedef struct packed {
    logic [15:0] transaction_id;
    logic [31:0] payload;
  } meta_packet_t;
  class pkg_templated_wrapper #(
    parameter int          DATA_W    = 32,
    parameter real         TIMEOUT   = 100.5,
    parameter bit [1:0][7:0] MASK_VAL = '{8'h0A, 8'h0B},
    parameter type         PAYLOAD_T = int
  );
    bit [DATA_W-1:0] val;
    real             max_time = TIMEOUT;
    bit [1:0][7:0]   mask     = MASK_VAL;
    PAYLOAD_T        payload_data;
    function new(bit [DATA_W-1:0] v = '0, PAYLOAD_T p = '0);
      val          = v;
      payload_data = p;
    endfunction
  endclass
endpackage
class subr_params_c #(
  parameter int       BUS_W   = 8,
  parameter type      DATA_T  = logic [7:0]
);
  static function bit [BUS_W-1:0] calculate_parity(DATA_T stream);
    bit [BUS_W-1:0] parity_accum = '0;
    for (int idx = 0; idx < $bits(DATA_T); idx++) begin
      parity_accum = parity_accum ^ stream[idx];
    end
    return parity_accum;
  endfunction
  static task monitor_bus(input DATA_T stream, output bit is_active);
    is_active = (stream != '0);
    #1ps;
  endtask
endclass
program prog_engine #(
  parameter int          PROG_W      = 16,
  parameter real         PROG_SAMPLE = 45.75,
  parameter bit [1:0][7:0] PROG_ARRAY  = '{8'hAA, 8'hBB},
  parameter type         PROG_TYPE   = infra_pkg::meta_packet_t
) (
  input  logic clk,
  output bit   program_done
);
  `SVTEST_INIT2
  logic [PROG_W-1:0] prog_vector;
  PROG_TYPE          prog_struct_var;
  real               prog_stored_real  = PROG_SAMPLE;
  bit [1:0][7:0]     prog_stored_array = PROG_ARRAY;
  initial begin
    program_done = 1'b0;
    #20ns;
    `SVTEST_CHECK_TRACK(0, ($bits(prog_vector) == 16), "Program value parameter width customization failed")
    `SVTEST_CHECK_TRACK(1, (prog_stored_real == 45.75), "Program real constant parameter override failed")
    `SVTEST_CHECK_TRACK(2, (prog_stored_array[1] === 8'hAA && prog_stored_array[0] === 8'hBB), "Program array parameter layout broke")
    `SVTEST_CHECK_TRACK(3, ($bits(prog_struct_var) == 48), "Program type parameter structural sizing failed")
    program_done = 1'b1;
    wait (test_tracker_pkg::top_done == 1'b1);
  end
endprogram
module tb_prog_params;
  `SVTEST_INIT2
  logic tb_clk = 1'b0;
  wire  tb_prog_done;
  always #5ns tb_clk = ~tb_clk;
  prog_engine #(
    .PROG_W(16),
    .PROG_SAMPLE(45.75),
    .PROG_ARRAY('{8'hAA, 8'hBB}),
    .PROG_TYPE(infra_pkg::meta_packet_t)
  ) u_prog_engine (
    .clk(tb_clk),
    .program_done(tb_prog_done)
  );
  initial begin
    infra_pkg::pkg_templated_wrapper #(
      .DATA_W(64),
      .TIMEOUT(12.34),
      .MASK_VAL('{8'hFF, 8'h00}),
      .PAYLOAD_T(infra_pkg::meta_packet_t)
    ) pkg_obj;
    typedef subr_params_c #(
      .BUS_W(4),
      .DATA_T(logic [31:0])
    ) custom_subroutine_t;
    bit [3:0] parity_out;
    bit       activity_out;
    pkg_obj = new(64'h1234567890ABCDEF, '{transaction_id: 16'hFFFF, payload: 32'h55555555});
    wait (tb_prog_done == 1'b1);
    #1ps;
    `SVTEST_CHECK_TRACK(4, ($bits(pkg_obj.val) == 64), "Package embedded class value parameter width override dropped")
    `SVTEST_CHECK_TRACK(4, (pkg_obj.max_time == 12.34), "Package embedded class real parameter override dropped")
    `SVTEST_CHECK_TRACK(4, (pkg_obj.mask[1] === 8'hFF && pkg_obj.mask[0] === 8'h00), "Package embedded class array parameter mapping broken")
    `SVTEST_CHECK_TRACK(5, ($bits(pkg_obj.payload_data) == 48), "Package embedded class type parameter sizing dropped")
    parity_out = custom_subroutine_t::calculate_parity(32'hFFFF_FFFF);
    `SVTEST_CHECK_TRACK(6, (parity_out == 4'h0), "Parameterized function bit processing boundary failed")
    custom_subroutine_t::monitor_bus(32'hA5A5_A5A5, activity_out);
    `SVTEST_CHECK_TRACK(7, (activity_out == 1'b1), "Parameterized task execution validation dropped")
    `SVTEST_PASSFAIL2
    #1ps;
    test_tracker_pkg::top_done = 1'b1;
  end
endmodule
