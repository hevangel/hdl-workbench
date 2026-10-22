`timescale 1ns / 1ps
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH
`define SVTEST_INIT \
int failures = 0;
`define SVTEST_CHECK(expr, msg) \
if (!(expr)) begin \
failures++; \
$display("FAIL: %s at time %0t ps", msg, $time); \
end
`define SVTEST_PASSFAIL \
if (failures == 0) begin \
$display("TEST_PASS"); \
end else begin \
$display("TEST_FAIL count=%0d", failures); \
$fatal(1); \
end
`endif

typedef struct packed {
  logic        vld;
  logic [3:0]  slot_id;
  logic [15:0] tag;
} slot_req_t;

module slot_queue #(
    parameter DEPTH = 8,
    parameter PTR_W = $clog2(DEPTH)
) (
    input logic gclk,
    input logic rst_g,
    input slot_req_t slot_req,
    input logic        grant,
    input logic [15:0] sel_d2,
    output logic [15:0] slot_rdy,
    output logic [15:0] full
);
  slot_req_t             squeue        [16] [DEPTH];
  logic                  sq_wr         [16];
  logic                  sq_rd         [16];
  logic      [PTR_W-1:0] wptr          [16];
  logic      [PTR_W-1:0] rptr          [16];
  logic      [  PTR_W:0] level         [16];
  logic      [PTR_W-1:0] wptr_nxt      [16];
  logic      [PTR_W-1:0] rptr_nxt      [16];
  logic      [  PTR_W:0] level_nxt     [16];
  integer                cfg_q_depth;
  initial cfg_q_depth = DEPTH;

  always_comb begin
    slot_rdy = ~full;
    for (int i = 0; i < 16; i++) begin
      sq_wr[i] = slot_rdy[i] && slot_req.vld && (slot_req.slot_id == i);
      sq_rd[i] = grant && sel_d2[i];
      wptr_nxt[i] = wptr[i] + sq_wr[i];
      rptr_nxt[i] = rptr[i] + sq_rd[i];
      level_nxt[i] = level[i] + sq_wr[i] - sq_rd[i];
    end
  end

  always_ff @(posedge gclk) begin
    for (int i = 0; i < 16; i++) begin
      if (sq_wr[i]) begin
        squeue[i][wptr[i]] <= slot_req;
      end
    end
  end

  always_ff @(posedge gclk) begin
    if (rst_g) begin
      for (int i = 0; i < 16; i++) begin
        wptr[i]  <= '0;
        rptr[i]  <= '0;
        level[i] <= '0;
        full[i]  <= '0;
      end
    end else begin
      for (int i = 0; i < 16; i++) begin
        if (sq_wr[i]) wptr[i] <= wptr_nxt[i];
        if (sq_rd[i]) rptr[i] <= rptr_nxt[i];
        if (sq_wr[i] || sq_rd[i]) begin
          level[i] <= level_nxt[i];
          full[i]  <= (level_nxt[i] == cfg_q_depth);
        end
      end
    end
  end
endmodule

module tb;
   `SVTEST_INIT
   localparam DEPTH   = 8;
   localparam LEVEL_W = $clog2(DEPTH)+1;
   logic clk;
   logic rst;
   slot_req_t slot_req;
   logic grant;
   logic [15:0] sel_d2;
   logic [15:0] slot_rdy;
   logic [15:0] full;

   slot_queue #(
      .DEPTH(DEPTH)
   ) dut (
      .gclk   (clk),
      .rst_g  (rst),
      .slot_req    (slot_req),
      .grant       (grant),
      .sel_d2 (sel_d2),
      .slot_rdy    (slot_rdy),
      .full        (full)
   );

   logic [31:0] lfsr;
   function automatic [31:0] lfsr_next(
      input [31:0] cur
   );
      lfsr_next = {
         cur[30:0],
         cur[31] ^ cur[21] ^ cur[1] ^ cur[0]
      };
   endfunction

   always @(posedge clk) begin
      if(rst)
         lfsr <= 32'h1ACE_B00C;
      else
         lfsr <= lfsr_next(lfsr);
   end

   logic [LEVEL_W-1:0] exp_level [16];
   logic               exp_full  [16];

   initial clk = 0;
   always #5 clk = ~clk;

   initial begin
      rst = 1;
      slot_req    = '0;
      grant       = 0;
      sel_d2 = 0;
      repeat(5)
         @(posedge clk);
      rst = 0;
   end

   initial begin
      wait(!rst);
      repeat(50)
         @(posedge clk);
      forever begin
         @(negedge clk);
         slot_req.vld     <= lfsr[0];
         slot_req.slot_id <= lfsr[4:1];
         slot_req.tag     <= lfsr[20:5];
         grant <= lfsr[21];
         sel_d2 <= '0;
         if(lfsr[21])
            sel_d2[lfsr[25:22]] <= 1'b1;
      end
   end

   always @(posedge clk) begin
      if(rst) begin
         for(int i=0;i<16;i++) begin
            exp_level[i] <= '0;
            exp_full[i]  <= 1'b0;
         end
      end
      else begin
         for(int i=0;i<16;i++) begin
            bit wr;
            bit rd;
            logic [LEVEL_W-1:0] next_level;
            wr =
               (~exp_full[i]) &&
               slot_req.vld &&
               (slot_req.slot_id == i);
            rd =
               grant &&
               sel_d2[i];
            next_level =
               exp_level[i] + wr - rd;
            exp_level[i] <= next_level;
            exp_full[i] <=
               (next_level == DEPTH);
         end
      end
   end

   always @(posedge clk) begin
      if(!rst) begin
         for(int i=0;i<16;i++) begin
            `SVTEST_CHECK(
               slot_rdy[i] === ~full[i],
               $sformatf(
                  "slot_rdy/full mismatch slot=%0d",
                  i
               )
            )
            `SVTEST_CHECK(
               dut.level[i] === exp_level[i],
               $sformatf(
                  "level mismatch slot=%0d exp=%0d got=%0d",
                  i,
                  exp_level[i],
                  dut.level[i]
               )
            )
            `SVTEST_CHECK(
               full[i] === exp_full[i],
               $sformatf(
                  "full mismatch slot=%0d exp=%0d got=%0d",
                  i,
                  exp_full[i],
                  full[i]
               )
            );
         end
      end
   end

   task automatic fill_slot3();
      for(int k=0;k<32;k++) begin
         @(negedge clk);
         slot_req.vld     <= 1'b1;
         slot_req.slot_id <= 4'd3;
         slot_req.tag     <= k;
         grant            <= 1'b0;
         sel_d2      <= '0;
      end
   endtask

   initial begin
      wait(!rst);
      repeat(500)
         @(posedge clk);
      fill_slot3();
      repeat(200)
         @(posedge clk);
      $display("");
      $display("-------------------------------------");
      $display("Deterministic LFSR regression complete");
      $display("-------------------------------------");
      $display("");
      `SVTEST_PASSFAIL
      $finish;
   end
endmodule
