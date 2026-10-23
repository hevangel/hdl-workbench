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

module comb_lrm_dut (
    input  logic       a, input  logic b, input  logic c, input  logic func_in,
    output logic       out_simple,
    output logic [1:0] out_complex,
    output logic       out_from_func,
    output int         time_0_trigger_count
);
  logic internal_func_dep;
  function automatic logic eval_with_hidden_dep(input logic p1);
    return p1 ^ internal_func_dep;
  endfunction
  always_comb begin
    time_0_trigger_count++;
    out_simple = a & b;
    if (c) begin
      out_complex = 2'b11;
    end else begin
      out_complex = {a, b};
    end
    out_from_func = eval_with_hidden_dep(func_in);
  end
endmodule
module tb_comb_lrm;
  `SVTEST_INIT
  logic       tb_a;
  logic       tb_b;
  logic       tb_c;
  logic       tb_func_in;
  wire        tb_out_simple;
  wire  [1:0] tb_out_complex;
  wire        tb_out_from_func;
  wire  [31:0] tb_time_0_trigger_count;
  comb_lrm_dut dut (
    .a(tb_a), .b(tb_b), .c(tb_c), .func_in(tb_func_in),
    .out_simple(tb_out_simple), .out_complex(tb_out_complex),
    .out_from_func(tb_out_from_func), .time_0_trigger_count(tb_time_0_trigger_count)
  );
  initial begin
    #0;
    `SVTEST_CHECK((tb_time_0_trigger_count == 1), "LRM 9.2.2.2.1: always_comb failed to execute at time 0")
    tb_a       = 1'b0;
    tb_b       = 1'b0;
    tb_c       = 1'b0;
    tb_func_in = 1'b0;
    dut.internal_func_dep = 1'b0;
    #1;
    $display("TC2: Verifying basic variable automatic sensitivity...");
    tb_a = 1'b1;
    tb_b = 1'b1;
    #1;
    `SVTEST_CHECK((tb_out_simple === 1'b1), "Sensitivity List: Change on 'a' or 'b' missed")
    `SVTEST_CHECK((tb_out_complex === 2'b11), "Sensitivity List: Complex block side-effect evaluation missed")
    tb_c = 1'b0;
    #1;
    `SVTEST_CHECK((tb_out_complex === 2'b11), "Sensitivity List: Branch change toggle missed")
    tb_b = 1'b0;
    #1;
    `SVTEST_CHECK((tb_out_complex === 2'b10), "Sensitivity List: Dependent evaluation missed")
    $display("TC3: Verifying implicit function dependency sensitivity...");
    tb_func_in = 1'b1;
    #1;
    `SVTEST_CHECK((tb_out_from_func === 1'b1), "Function Trace: Explicit parameter toggle failed to trigger")
    dut.internal_func_dep = 1'b1;
    #1;
    `SVTEST_CHECK((tb_out_from_func === 1'b0), "LRM 9.2.2.2.2: Variable inside function scope missed sensitivity inclusion")
    $display("TC4: Verifying multi-variable simultaneous adjustments...");
    tb_a = 1'b0;
    tb_b = 1'b1;
    tb_c = 1'b1;
    #1;
    `SVTEST_CHECK((tb_out_simple  === 1'b0), "Multi-adjust: out_simple evaluation error")
    `SVTEST_CHECK((tb_out_complex === 2'b11), "Multi-adjust: out_complex evaluation error")
    $display("--- IEEE 1800-2017 Sec 9.2.2.2 Complete ---");
    `SVTEST_PASSFAIL
  end
endmodule
