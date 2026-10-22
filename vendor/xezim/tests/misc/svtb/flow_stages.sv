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

module flow_stage #(
   parameter param_DW = 1, parameter param_VW = 1, parameter param_BC = 1,
   parameter param_RS = 1, parameter param_XD = 1, parameter param_RD = 0
) (
   input  logic                  clk, input  logic rst,
   input  logic [param_VW-1:0]   sig_ivld, output logic sig_inxt,
   input  logic [param_DW-1:0]   sig_idat,
   output logic [param_VW-1:0]   sig_ovld, input logic sig_onxt,
   output logic [param_DW-1:0]   sig_odat
);
   logic  sig_ctrl_load;
   logic  sig_ctrl_stall = 0;
   logic [param_VW-1:0] sig_masked_ivld;
   logic             sig_masked_inxt;
   assign sig_masked_ivld = sig_ctrl_stall ? 0 : sig_ivld;
   assign sig_inxt        = sig_ctrl_stall ? 0 : sig_masked_inxt;
   assign sig_masked_inxt = param_BC ? (sig_onxt | !(|sig_ovld)) : sig_onxt;
   assign sig_ctrl_load   = (|sig_masked_ivld) & sig_masked_inxt;
   always_ff @(posedge clk) begin
     if (rst) begin
       sig_ovld <= 0;
     end else begin
       sig_ovld <= sig_masked_inxt ? sig_masked_ivld : sig_ovld;
     end
     sig_odat <= sig_ctrl_load ? sig_idat : sig_odat;
     if (param_RD && rst) sig_odat <= 0;
   end
endmodule
module tb_flow_stages;
   `SVTEST_INIT
   logic clk;
   logic rst;
   initial begin
      clk = 0;
      forever #5 clk = ~clk;
   end
   localparam int NUM_INSTANCES = 5;
   logic [NUM_INSTANCES-1:0] [3:0]  ivld, ovld;
   logic [NUM_INSTANCES-1:0]        inxt, onxt;
   logic [NUM_INSTANCES-1:0] [63:0] idat, odat;
   logic [63:0] expected_queue [NUM_INSTANCES-1:0] [$];
   flow_stage #(.param_DW(16), .param_VW(1), .param_BC(1), .param_RS(0)) u_stage0 (
      .clk(clk), .rst(rst),
      .sig_ivld(ivld[0:0]), .sig_inxt(inxt[0]), .sig_idat(idat[0][15:0]),
      .sig_ovld(ovld[0:0]), .sig_onxt(onxt[0]), .sig_odat(odat[0][15:0])
   );
   flow_stage #(.param_DW(32), .param_VW(2), .param_BC(1), .param_RS(0)) u_stage1 (
      .clk(clk), .rst(rst),
      .sig_ivld(ivld[1][1:0]), .sig_inxt(inxt[1]), .sig_idat(idat[1][31:0]),
      .sig_ovld(ovld[1][1:0]), .sig_onxt(onxt[1]), .sig_odat(odat[1][31:0])
   );
   flow_stage #(.param_DW(8), .param_VW(1), .param_BC(0), .param_RS(0)) u_stage2 (
      .clk(clk), .rst(rst),
      .sig_ivld(ivld[2][0:0]), .sig_inxt(inxt[2]), .sig_idat(idat[2][7:0]),
      .sig_ovld(ovld[2][0:0]), .sig_onxt(onxt[2]), .sig_odat(odat[2][7:0])
   );
   flow_stage #(.param_DW(64), .param_VW(1), .param_BC(1), .param_RS(0)) u_stage3 (
      .clk(clk), .rst(rst),
      .sig_ivld(ivld[3][0:0]), .sig_inxt(inxt[3]), .sig_idat(idat[3][63:0]),
      .sig_ovld(ovld[3][0:0]), .sig_onxt(onxt[3]), .sig_odat(odat[3][63:0])
   );
   flow_stage #(.param_DW(16), .param_VW(1), .param_BC(1), .param_RS(0), .param_RD(1)) u_stage4 (
      .clk(clk), .rst(rst),
      .sig_ivld(ivld[4][0:0]), .sig_inxt(inxt[4]), .sig_idat(idat[4][15:0]),
      .sig_ovld(ovld[4][0:0]), .sig_onxt(onxt[4]), .sig_odat(odat[4][15:0])
   );
   int param_vw [NUM_INSTANCES] = '{1, 2, 1, 1, 1};
   int param_dw [NUM_INSTANCES] = '{16, 32, 8, 64, 16};
   always @(posedge clk) begin : checker_block
      if (!rst) begin
         for (int i = 0; i < NUM_INSTANCES; i++) begin
            automatic logic [63:0] masked_idat;
            automatic logic [63:0] exp_val;
            automatic logic [63:0] act_val;
            if ((|(ivld[i] & ((1<<param_vw[i])-1))) && inxt[i]) begin
               masked_idat = idat[i] & ((64'h1 << param_dw[i]) - 1);
               expected_queue[i].push_back(masked_idat);
            end
            if ((|(ovld[i] & ((1<<param_vw[i])-1))) && onxt[i]) begin
               `SVTEST_CHECK(expected_queue[i].size() > 0,
                  $sformatf("[INST %0d] Scoreboard Error: Unexpected output transaction detected!", i))
               if (expected_queue[i].size() > 0) begin
                  exp_val = expected_queue[i].pop_front();
                  act_val = odat[i] & ((64'h1 << param_dw[i]) - 1);
                  `SVTEST_CHECK(act_val === exp_val,
                     $sformatf("[INST %0d] DATA MISMATCH! Expected: %h, Actual: %h", i, exp_val, act_val))
               end
            end
         end
      end
   end
   initial begin
      rst     = 1'b1;
      ivld    = '0;
      onxt    = '0;
      idat    = '0;
      repeat(5) @(posedge clk);
      @(negedge clk);
      rst     = 1'b0;
      repeat(100) begin
         @(negedge clk);
         for (int i = 0; i < NUM_INSTANCES; i++) begin
            if ($urandom_range(0, 9) < 7) begin
               ivld[i] = $urandom & ((1 << param_vw[i]) - 1);
            end else begin
               ivld[i] = '0;
            end
            idat[i] = {$urandom, $urandom} & ((64'h1 << param_dw[i]) - 1);
            onxt[i] = ($urandom_range(0, 9) < 8);
         end
      end
      @(negedge clk);
      ivld = '0;
      onxt = '1;
      fork
         begin : wait_flush
            for (int i = 0; i < NUM_INSTANCES; i++) begin
               while (expected_queue[i].size() > 0) begin
                  @(posedge clk);
               end
            end
         end
      join_any
      repeat(5) @(posedge clk);
      #1;
      `SVTEST_PASSFAIL
      $finish;
   end
endmodule
