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

module cool_tracker #(
   parameter NUM_LANES = 16
)(
   input  logic         clk_i,
   input  logic         rst_i,
   input  logic         issue_vld_i,
   input  logic         issue_wr_i,
   input  logic [3:0]   issue_slot_i,
   input  logic [15:0]  rsp_enable_i,
   input  logic [15:0]  reload_wr_i,
   input  logic [15:0]  reload_rd_i,
   output logic [15:0]  slot_ready_o
);
   logic [15:0]         slot_match;
   logic [15:0][15:0]   cooldown_cnt;
   logic [15:0]         slot_active;
   always_comb begin
      for (int i=0;i<16;i++) begin
         slot_match[i] = (issue_slot_i == i) && issue_vld_i;
         slot_ready_o[i] = (cooldown_cnt[i] <= 1) && !slot_match[i] && slot_active[i];
      end
   end
   always_ff @(posedge clk_i) begin
      if (rst_i) begin
         cooldown_cnt <= '{default:'0};
         slot_active  <= '{default:1'b1};
      end
      else begin
         for (int i=0;i<16;i++) begin
            if (slot_match[i]) begin
               cooldown_cnt[i] <= (issue_wr_i) ? reload_wr_i : reload_rd_i;
               slot_active[i] <= 1'b0;
            end
            else begin
               cooldown_cnt[i] <= (cooldown_cnt[i] > 0) ? cooldown_cnt[i]-1'b1 : cooldown_cnt[i];
               slot_active[i] <= rsp_enable_i[i] ? 1'b1 : slot_active[i];
            end
         end
      end
   end
endmodule
module tb_cool;
   `SVTEST_INIT
   logic        clk;
   logic        rst;
   logic        issue_vld;
   logic        issue_wr;
   logic [3:0]  issue_slot;
   logic [15:0] rsp_enable;
   logic [15:0] reload_wr;
   logic [15:0] reload_rd;
   logic [15:0] slot_ready;
   cool_tracker dut (
      .clk_i          (clk),
      .rst_i          (rst),
      .issue_vld_i    (issue_vld),
      .issue_wr_i     (issue_wr),
      .issue_slot_i   (issue_slot),
      .rsp_enable_i   (rsp_enable),
      .reload_wr_i    (reload_wr),
      .reload_rd_i    (reload_rd),
      .slot_ready_o   (slot_ready)
   );
   logic [31:0] lfsr;
   function automatic [31:0] lfsr_next(input [31:0] cur);
      lfsr_next = { cur[30:0], cur[31] ^ cur[21] ^ cur[1] ^ cur[0] };
   endfunction
   logic [15:0][15:0] exp_cnt;
   logic [15:0]       exp_enable;
   logic [15:0]       exp_ready;
   initial clk = 0;
   always #5 clk = ~clk;
   initial begin
      rst         = 1;
      issue_vld   = 0;
      issue_wr    = 0;
      issue_slot  = 0;
      rsp_enable  = 0;
      reload_wr   = 16'd5;
      reload_rd   = 16'd3;
      lfsr        = 32'h1ACE_B00C;
      repeat (5) @(posedge clk);
      rst = 0;
   end
   always @(posedge clk) begin
      if (rst)
         lfsr <= 32'h1ACE_B00C;
      else
         lfsr <= lfsr_next(lfsr);
   end
   initial begin
      wait (!rst);
      repeat (50) @(posedge clk);
      forever begin
         @(negedge clk);
         issue_vld  <= lfsr[0];
         issue_wr   <= lfsr[1];
         issue_slot <= lfsr[5:2];
         rsp_enable <= lfsr[21:6];
         reload_wr  <= {8'h00,lfsr[29:22]};
         reload_rd  <= {8'h00,lfsr[21:14]};
      end
   end
   always @(posedge clk) begin
      if (rst) begin
         exp_cnt    <= '{default:'0};
         exp_enable <= '1;
      end
      else begin
         for (int i=0;i<16;i++) begin
            bit hit;
            hit = issue_vld && (issue_slot == i);
            if (hit) begin
               exp_cnt[i] <= issue_wr ? reload_wr : reload_rd;
               exp_enable[i] <= 1'b0;
            end
            else begin
               exp_cnt[i] <= (exp_cnt[i] > 0) ? exp_cnt[i]-1'b1 : exp_cnt[i];
               exp_enable[i] <= rsp_enable[i] ? 1'b1 : exp_enable[i];
            end
         end
      end
   end
   always_comb begin
      for (int i=0;i<16;i++) begin
         exp_ready[i] = (exp_cnt[i] <= 1) && !(issue_vld && (issue_slot == i)) && exp_enable[i];
      end
   end
   always @(posedge clk) begin
      if (!rst) begin
         for (int i=0;i<16;i++) begin
            `SVTEST_CHECK(dut.cooldown_cnt[i] === exp_cnt[i], $sformatf("cooldown count mismatch slot=%0d", i))
            `SVTEST_CHECK(dut.slot_active[i] === exp_enable[i], $sformatf("slot active mismatch slot=%0d", i))
            `SVTEST_CHECK(slot_ready[i] === exp_ready[i], $sformatf("slot_ready mismatch slot=%0d", i))
         end
      end
   end
   initial begin
      wait(!rst);
      repeat(2000) @(posedge clk);
      @(negedge clk);
      issue_vld  <= 1'b1;
      issue_wr   <= 1'b1;
      issue_slot <= 4'd7;
      reload_wr  <= 16'd20;
      @(posedge clk);
      `SVTEST_CHECK(slot_ready[7] == 1'b0, "slot7 should not be ready after issue")
      repeat(200) @(posedge clk);
      `SVTEST_PASSFAIL
      $finish;
   end
endmodule
