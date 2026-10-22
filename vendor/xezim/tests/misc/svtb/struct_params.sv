`define SVTEST_INIT3 \
  int failures = 0; \
  bit [63:0] test_status = '1;
`define SVTEST_CHECK_TRACK(index, expr, msg) \
  if (!(expr)) begin \
    failures++; \
    test_status[index] = 1'b0; \
    $display("FAIL: %s at time %0t ps", msg, $time); \
  end
`define SVTEST_PASSFAIL3 \
  if (failures == 0) begin \
    $display("TEST_PASS"); \
  end else begin \
    $display("TEST_FAIL count=%0d", failures); \
    $fatal(1); \
  end
module consumer #(
  parameter int STRUCT_SIZE_T = 1
) (
  input  logic [STRUCT_SIZE_T-1:0] packed_struct_flat,
  output logic [STRUCT_SIZE_T-1:0] echo_packed_flat,
  output logic [31:0]             struct_size_param_out
);
  assign echo_packed_flat     = packed_struct_flat;
  assign struct_size_param_out = STRUCT_SIZE_T;
endmodule
module design_module #(
  parameter int A = 8, parameter int B = 4, parameter int C = 16,
  parameter int D = 2, parameter int E = 3
) (
  output logic [31:0] struct_size_report,
  output logic [0:0]  struct_size_match_flag,
  output logic [A-1:0] out_f1,
  output logic [ (A*B)-1:0 ] out_f2,
  output logic [ (C/2)-1:0 ] out_f3,
  output logic [D-1:0] out_f4,
  output logic [E-1:0] out_f5,
  output logic [ ($bits({{A{1'b0}}, {(A*B){1'b0}}, {(C/2){1'b0}}, {D{1'b0}}, {E{1'b0}}})-1):0 ] packed_out
);
  localparam int LP1 = A * B;
  localparam int LP2 = C / 2;
  localparam int LP3 = LP1 + LP2;
  localparam int TOTAL_BITS = A + LP1 + LP2 + D + E;
  typedef struct packed {
    logic [A-1:0]       f1;
    logic [LP1-1:0]     f2;
    logic [LP2-1:0]     f3;
    logic [D-1:0]       f4;
    logic [E-1:0]       f5;
  } struct_type_t;
  localparam int STRUCT_BITS = $bits(struct_type_t);
  struct_type_t s_inst;
  logic [STRUCT_BITS-1:0] packed_s;
  logic [STRUCT_BITS-1:0] consumer_echo;
  logic [31:0]            consumer_struct_size_param;
  initial begin
    s_inst.f1 = {A{1'b1}};
    for (int i = 0; i < LP1; i++) s_inst.f2[i] = (i % 2) ? 1'b1 : 1'b0;
    s_inst.f3 = {LP2{1'b1}};
    s_inst.f4 = {D{1'b1}};
    s_inst.f5 = {E{1'b0}};
    packed_s = { s_inst.f1, s_inst.f2, s_inst.f3, s_inst.f4, s_inst.f5 };
  end
  assign out_f1 = s_inst.f1;
  assign out_f2 = s_inst.f2;
  assign out_f3 = s_inst.f3;
  assign out_f4 = s_inst.f4;
  assign out_f5 = s_inst.f5;
  consumer #(
    .STRUCT_SIZE_T(STRUCT_BITS)
  ) u_cons (
    .packed_struct_flat(packed_s),
    .echo_packed_flat(consumer_echo),
    .struct_size_param_out(consumer_struct_size_param)
  );
  assign struct_size_report = consumer_struct_size_param;
  assign packed_out = consumer_echo;
  assign struct_size_match_flag = (STRUCT_BITS == consumer_struct_size_param) ? 1'b1 : 1'b0;
endmodule
module tb_combined_struct_test;
  `SVTEST_INIT3
  localparam int TB_A = 8;
  localparam int TB_B = 4;
  localparam int TB_C = 16;
  localparam int TB_D = 2;
  localparam int TB_E = 3;
  localparam int EXP_LP1 = TB_A * TB_B;
  localparam int EXP_LP2 = TB_C / 2;
  localparam int EXP_TOTAL_BITS = TB_A + EXP_LP1 + EXP_LP2 + TB_D + TB_E;
  wire [31:0] dut_struct_size_report;
  wire        dut_struct_size_match_flag;
  wire [TB_A-1:0] dut_f1;
  wire [EXP_LP1-1:0] dut_f2;
  wire [EXP_LP2-1:0] dut_f3;
  wire [TB_D-1:0] dut_f4;
  wire [TB_E-1:0] dut_f5;
  wire [EXP_TOTAL_BITS-1:0] dut_packed_out;
  design_module #(
    .A(TB_A), .B(TB_B), .C(TB_C), .D(TB_D), .E(TB_E)
  ) dut (
    .struct_size_report(dut_struct_size_report),
    .struct_size_match_flag(dut_struct_size_match_flag),
    .out_f1(dut_f1),
    .out_f2(dut_f2),
    .out_f3(dut_f3),
    .out_f4(dut_f4),
    .out_f5(dut_f5),
    .packed_out(dut_packed_out)
  );
  logic [EXP_TOTAL_BITS-1:0] expected_packed;
  initial begin
    logic [TB_A-1:0] exp_f1 = {TB_A{1'b1}};
    logic [EXP_LP1-1:0] exp_f2;
    logic [EXP_LP2-1:0] exp_f3 = {EXP_LP2{1'b1}};
    logic [TB_D-1:0] exp_f4 = {TB_D{1'b1}};
    logic [TB_E-1:0] exp_f5 = {TB_E{1'b0}};
    for (int i = 0; i < EXP_LP1; i++) exp_f2[i] = (i % 2) ? 1'b1 : 1'b0;
    expected_packed = { exp_f1, exp_f2, exp_f3, exp_f4, exp_f5 };
  end
  initial begin
    logic [EXP_LP1-1:0] tb_exp_f2;
    #5ns;
    `SVTEST_CHECK_TRACK(0, (dut_struct_size_report == EXP_TOTAL_BITS), "Consumer parameter STRUCT_SIZE_T does not match expected struct width")
    `SVTEST_CHECK_TRACK(1, (dut_struct_size_match_flag == 1'b1), "Design-module reported struct width mismatch (STRUCT_BITS vs consumer param)")
    `SVTEST_CHECK_TRACK(2, (dut_packed_out === expected_packed), "Packed struct contents do not match expected pattern")
    `SVTEST_CHECK_TRACK(3, (dut_f1 === {TB_A{1'b1}}), "Field f1 pattern mismatch")
    for (int i = 0; i < EXP_LP1; i++) tb_exp_f2[i] = (i % 2) ? 1'b1 : 1'b0;
    `SVTEST_CHECK_TRACK(4, (dut_f2 === tb_exp_f2), "Field f2 alternating pattern mismatch")
    `SVTEST_CHECK_TRACK(5, (dut_f3 === {EXP_LP2{1'b1}}), "Field f3 pattern mismatch")
    `SVTEST_CHECK_TRACK(6, (dut_f4 === {TB_D{1'b1}}), "Field f4 pattern mismatch")
    `SVTEST_CHECK_TRACK(7, (dut_f5 === {TB_E{1'b0}}), "Field f5 pattern mismatch")
    `SVTEST_CHECK_TRACK(8, (EXP_LP1 == (TB_A * TB_B)), "LP1 arithmetic mismatch")
    `SVTEST_CHECK_TRACK(9, (EXP_LP2 == (TB_C / 2)), "LP2 arithmetic mismatch")
    `SVTEST_CHECK_TRACK(10, (EXP_TOTAL_BITS == (TB_A + EXP_LP1 + EXP_LP2 + TB_D + TB_E)), "TOTAL_BITS arithmetic mismatch")
    `SVTEST_PASSFAIL3
    $finish;
  end
endmodule
