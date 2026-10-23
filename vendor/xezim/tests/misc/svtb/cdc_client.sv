package sync_pkg ;

   typedef struct packed { logic [9:0] p3; logic [9:0] p2; logic [9:0] p1; logic [9:0] p0; } quad_sync_t;

endpackage: sync_pkg

module gray_sync #(parameter SYNC_W = 3) (
   input  logic                      ZrhWtqi,
   input  logic [(SYNC_W - 1):0] qzPuWyvh,
   input  logic                      OOhkfwuz,
   output logic [(SYNC_W - 1):0] Pj0Mp9ooY,
   input  logic [(SYNC_W - 1):0] pT64cf14TWaRtWW4MhH,
   output logic [(SYNC_W - 1):0] srLyKkzFTbvYqS8E8c
);

   // Perform binary to gray code coversion first & flop in source clock domain
   logic [(SYNC_W - 1):0] Fo9XBT9bKO01f;
   logic [(SYNC_W - 1):0] jIyTiHMlIEjUwyjM;

   assign jIyTiHMlIEjUwyjM = (qzPuWyvh >> 1);
   assign Fo9XBT9bKO01f = jIyTiHMlIEjUwyjM ^ qzPuWyvh;

   always_ff @(posedge ZrhWtqi) begin
      srLyKkzFTbvYqS8E8c <= Fo9XBT9bKO01f;
   end

   logic [(SYNC_W - 1):0] dFLOfp0r5k8Ks;
   generate
      for(genvar m = 0; m < SYNC_W; m = m + 1) begin: yXpTIrb78qBrKZ
         assign dFLOfp0r5k8Ks[m] = ^(pT64cf14TWaRtWW4MhH[(SYNC_W - 1):m]);
      end
   endgenerate

   assign   Pj0Mp9ooY = dFLOfp0r5k8Ks;

endmodule


module credit_sync #(parameter NPTR   = 4,parameter INCR_W = 4, parameter SYNC_W = 8) (
   input  logic                  ZrhWtqi,
   input  logic                  BmclJAsbT,
   input  logic [INCR_W-1:0] Q2HdxbEc,
   input  logic                  OOhkfwuz,
   input  logic                  sM5EXEMYs8,
   output logic [INCR_W-1:0] WN1ymAOmS, //lockstep_cg dest_clk

   output logic [NPTR-1:0][SYNC_W-1:0] iziBIr9Q8xkmfUdgn, //lockstep_cg src_clk
   input  logic [NPTR-1:0][SYNC_W-1:0] gXhJJZ4fHBUwp5EnWn
);

   localparam G5bxSrzG = (1 << INCR_W)-1;//input max increment is actually (1 << (INCR_WIDTH-1))

   logic [SYNC_W-1:0] Zo37URHa;
   logic [SYNC_W-1:0] B1Zt77KE;
   logic [SYNC_W-1:0] wSh0MuCB;
   logic [SYNC_W-1:0] vcwD2lQF9KuR[NPTR-1:0];
   logic [SYNC_W-1:0] pjcv4g0[NPTR-1:0];
   logic [SYNC_W-1:0] JDX1CVMM[NPTR-1:0];
   logic [SYNC_W-1:0] QMiK8nyWm3TZ[NPTR-1:0];
   logic [SYNC_W-1:0] MtEp2NGhI50BqN[NPTR-1:0];

   logic [SYNC_W-1:0] CJybuEFj;
   logic [NPTR-1:0]   aZlSgGqJWDL;
   logic [NPTR*2-1:0] dN0U0bl1Nwl;
   logic [NPTR-1:0]   ZkLo25m9CGU;

   logic [SYNC_W-1:0] ufQdiLe4NE0Jb;

   generate
      if(NPTR==1) begin : NAaQRFZzilk9
         always_comb begin
            CJybuEFj = B1Zt77KE-wSh0MuCB;
            aZlSgGqJWDL  = ((CJybuEFj == 0) | CJybuEFj[SYNC_W-1]) ? 0 : 1'h1;
            dN0U0bl1Nwl  = aZlSgGqJWDL;
            ZkLo25m9CGU  = dN0U0bl1Nwl[NPTR-1:0] | dN0U0bl1Nwl[NPTR*2-1:NPTR];
         end

         always @(posedge ZrhWtqi) begin
            B1Zt77KE <= (BmclJAsbT) ? 0 : B1Zt77KE + Q2HdxbEc;
            wSh0MuCB <= (BmclJAsbT) ? 0 : wSh0MuCB + ZkLo25m9CGU[0];
         end

         assign ufQdiLe4NE0Jb = MtEp2NGhI50BqN[0];
      end
      else if(NPTR==2) begin : kZki0vtGy4vz
         always_comb begin
            CJybuEFj = B1Zt77KE-wSh0MuCB;
            aZlSgGqJWDL  = ((CJybuEFj == 0) | CJybuEFj[SYNC_W-1]) ? 0 :
                         (CJybuEFj == 1) ? 2'h1 : 2'h3;
            dN0U0bl1Nwl  = aZlSgGqJWDL << wSh0MuCB[0];
            ZkLo25m9CGU  = dN0U0bl1Nwl[NPTR-1:0] | dN0U0bl1Nwl[NPTR*2-1:NPTR];
         end

         always @(posedge ZrhWtqi) begin
            B1Zt77KE <= (BmclJAsbT) ? 0 : B1Zt77KE + Q2HdxbEc;
            wSh0MuCB <= (BmclJAsbT) ? 0 : wSh0MuCB + ZkLo25m9CGU[0] + ZkLo25m9CGU[1];
         end

         assign ufQdiLe4NE0Jb = MtEp2NGhI50BqN[0] + MtEp2NGhI50BqN[1];
      end
      else if(NPTR==4) begin : jkidu7kBjbWq
         always_comb begin
            CJybuEFj = B1Zt77KE-wSh0MuCB;
            aZlSgGqJWDL  = ((CJybuEFj == 0) | CJybuEFj[SYNC_W-1]) ? 0 :
                         (CJybuEFj == 1) ? 4'h1 :
                         (CJybuEFj == 2) ? 4'h3 :
                         (CJybuEFj == 3) ? 4'h7 : 4'hf;
            dN0U0bl1Nwl  = aZlSgGqJWDL << wSh0MuCB[1:0];
            ZkLo25m9CGU  = dN0U0bl1Nwl[NPTR-1:0] | dN0U0bl1Nwl[NPTR*2-1:NPTR];
         end

         always @(posedge ZrhWtqi) begin
            B1Zt77KE <= (BmclJAsbT) ? 0 : B1Zt77KE + Q2HdxbEc;
            wSh0MuCB <= (BmclJAsbT) ? 0 : wSh0MuCB + ZkLo25m9CGU[0] + ZkLo25m9CGU[1] + ZkLo25m9CGU[2] + ZkLo25m9CGU[3];
         end

         assign ufQdiLe4NE0Jb = MtEp2NGhI50BqN[0] + MtEp2NGhI50BqN[1] + MtEp2NGhI50BqN[2] + MtEp2NGhI50BqN[3];
      end
      else if(NPTR==16) begin : yK93Oh8R4aSCc
         always_comb begin
            CJybuEFj = B1Zt77KE-wSh0MuCB;
            aZlSgGqJWDL  = ((CJybuEFj == 0) | CJybuEFj[SYNC_W-1]) ? 0 :
                        ({NPTR{1'b1}} >> (NPTR - ((CJybuEFj < NPTR) ? CJybuEFj : NPTR)));
            dN0U0bl1Nwl  = aZlSgGqJWDL << wSh0MuCB[1:0];
            ZkLo25m9CGU  = dN0U0bl1Nwl[NPTR-1:0] | dN0U0bl1Nwl[NPTR*2-1:NPTR];
         end

         always @(posedge ZrhWtqi) begin
            B1Zt77KE <= (BmclJAsbT) ? 0 : B1Zt77KE + Q2HdxbEc;
            wSh0MuCB <= (BmclJAsbT) ? 0 : wSh0MuCB + ZkLo25m9CGU[0] + ZkLo25m9CGU[1] + ZkLo25m9CGU[2] + ZkLo25m9CGU[3]
                                                + ZkLo25m9CGU[4] + ZkLo25m9CGU[5] + ZkLo25m9CGU[6] + ZkLo25m9CGU[7]
                                                + ZkLo25m9CGU[8] + ZkLo25m9CGU[9] + ZkLo25m9CGU[10] + ZkLo25m9CGU[11]
                                                + ZkLo25m9CGU[12] + ZkLo25m9CGU[13] + ZkLo25m9CGU[14] + ZkLo25m9CGU[15];
         end

         assign ufQdiLe4NE0Jb = MtEp2NGhI50BqN[0] + MtEp2NGhI50BqN[1] + MtEp2NGhI50BqN[2] + MtEp2NGhI50BqN[3]
                           + MtEp2NGhI50BqN[4] + MtEp2NGhI50BqN[5] + MtEp2NGhI50BqN[6] + MtEp2NGhI50BqN[7]
                           + MtEp2NGhI50BqN[8] + MtEp2NGhI50BqN[9] + MtEp2NGhI50BqN[10] + MtEp2NGhI50BqN[11]
                           + MtEp2NGhI50BqN[12] + MtEp2NGhI50BqN[13] + MtEp2NGhI50BqN[14] + MtEp2NGhI50BqN[15];
      end

      assign WN1ymAOmS = (Zo37URHa >= G5bxSrzG) ? G5bxSrzG : Zo37URHa;

      always_ff @ (posedge OOhkfwuz) begin
         if (sM5EXEMYs8) begin
            Zo37URHa <= 'd0;;
         end
         else begin
            Zo37URHa <= Zo37URHa + ufQdiLe4NE0Jb - WN1ymAOmS;
         end
      end

      for(genvar m = 0; m < NPTR; m++) begin : moQ0PYphgcuQaCP

         assign  vcwD2lQF9KuR[m] = (BmclJAsbT) ? 0 : pjcv4g0[m] + ZkLo25m9CGU[m];
         assign  MtEp2NGhI50BqN[m] = JDX1CVMM[m] - QMiK8nyWm3TZ[m];

         always_ff @ (posedge ZrhWtqi) begin
            pjcv4g0[m] <= vcwD2lQF9KuR[m];
         end

         always_ff @ (posedge OOhkfwuz) begin
            if (sM5EXEMYs8) begin
              QMiK8nyWm3TZ[m] <= 0;
            end
            else begin
              QMiK8nyWm3TZ[m] <= JDX1CVMM[m];
            end
         end

         gray_sync #(.SYNC_W(SYNC_W)) u_gray (
            .ZrhWtqi   (ZrhWtqi),
            .qzPuWyvh  (vcwD2lQF9KuR[m]),
            .OOhkfwuz  (OOhkfwuz),
            .Pj0Mp9ooY (JDX1CVMM[m]),
            .pT64cf14TWaRtWW4MhH(gXhJJZ4fHBUwp5EnWn[m]),
            .srLyKkzFTbvYqS8E8c (iziBIr9Q8xkmfUdgn[m])
         );

      end

   endgenerate

endmodule

module cdc_client import sync_pkg::* ;
   (/*AUTOARG*/
   // Outputs
   wen_n, addr_o, data_o,
   mask_o, sync_out, cred_incr,
   ptr_fan,
   // Inputs
   clk_src, rst_src, clk_dst, rst_dst,
   in_flat, sync_in
   );

   parameter IN_W = 256;                                  //AXI: 72 = 64 + 8(byte vld) + 1(last bit), Non AXI: no Byte vld, no last bit
   parameter ADDR_W      = 8;                                    //In rows of staging rows
   parameter PTR_W     = 8;                                    //In Lanes
   parameter Ma                 = 64;
   parameter NL             = 2;
   parameter INCR_W         = 1;
   localparam MASK_W           = 8*NL;
   localparam DATA_W2          = 64*NL;
   localparam LOG_NL        = $clog2(NL);
   localparam WEN_W            = NL;

   localparam UNUSED_LP   = 0;
   localparam DATA_W   = Ma;
   localparam MASK_W2   = (Ma/8);

   input  logic                                        clk_src;
   input  logic                                        rst_src;

   input  logic                                        clk_dst;
   input  logic                                        rst_dst;

   input  logic [IN_W-1:0]               in_flat;
   output logic [WEN_W-1:0]                           wen_n;
   output logic [ADDR_W-1:0]                    addr_o;
   output logic [DATA_W-1:0]                  data_o; // (NLANES+1): NLANES number of lane vld bits, +1 last bit (for both AXI and non-AXI clients)
   output logic [MASK_W2-1:0]                  mask_o; // (DW/8): byte mask + (byte mask of byte mask, if applicable); NLANES: lane vld mask; +1: last bit

   output quad_sync_t           sync_out;
   input  quad_sync_t           sync_in;

   output logic [INCR_W-1:0]                       cred_incr;
   output logic [3:0][9:0]                             ptr_fan;


   //Local Signals
   logic [IN_W-1:0]                       BbiwO5ARfgxO6Bewdgjo3;
   logic [NL-1:0]                                   j22r5RZ9Aec4l;
   logic [Ma-1:0]                                       eqOrL6s2n;

   logic [PTR_W:0]                             fYsZs52E4uCBc;
   logic [PTR_W:0]                             ZVinrHqN7VE;

   logic [LOG_NL:0]                                 W8pUYo0es;
   logic [INCR_W-1:0]                               a22aO48x4eVsBW;
   logic [3:0][9:0]                                     ptr_loc;

   logic [NL-1:0]                                   XUL;
   logic                                                w8JxCOz1;
   logic [Ma-1:0]                                       qSpj;
   logic [NL-1:0]                                   hEko7jtLxxmWEz;
   logic [(Ma/8)-1:0]                                   Lnpi4RWnDP;
   logic                                                k3I0Kwk7jw;
   logic [Ma-1:0]                                       rXb6Mt1Zpe2;
   logic                                                X8zUELUV;
   logic [Ma-1:0]                                       qWmdo6hK5;
   logic [LOG_NL-1:0]                               aywFyeD3;
   logic [NL-1:0]                                   HCS5CwLG;
   logic [Ma/8-1:0]                                     KQPEQPLJDd3vbWp;
   logic [Ma-1:0]                                       OOhuDz55Y78rH7;
   logic [NL-1:0]                                   hHxvMimagtwiq65;
   logic [(Ma/8)-1:0]                                   SKvxd0pCsRaiTpGr;

   logic [LOG_NL-1:0]                               msSQHxFww64e;
   logic [LOG_NL:0]                                 MAx;



   //Implementation

   always_ff @(posedge clk_src) begin
      BbiwO5ARfgxO6Bewdgjo3 <= in_flat;
   end

   always_comb begin
      eqOrL6s2n        = BbiwO5ARfgxO6Bewdgjo3[IN_W-1:NL];
      j22r5RZ9Aec4l    = BbiwO5ARfgxO6Bewdgjo3[NL-1:0];
      //wptr_incr        = $countbits(wdata_inp_vld, '1);
      W8pUYo0es             = 0;
      for(int m=0; m<NL; m++) begin
         W8pUYo0es         += j22r5RZ9Aec4l[m];
      end
      msSQHxFww64e     = W8pUYo0es-1;
      hEko7jtLxxmWEz   = ({NL{1'b1}} >> (NL - W8pUYo0es)) << (ZVinrHqN7VE & (NL-1));
      Lnpi4RWnDP       = ({MASK_W{1'b1}} >> (MASK_W - (W8pUYo0es << 3))) << ((ZVinrHqN7VE & (NL-1)) << 3);
      HCS5CwLG         = ({NL{1'b1}}) >> (NL - (ZVinrHqN7VE & (NL-1)));
      KQPEQPLJDd3vbWp  = ({MASK_W{1'b1}}) >> (NL - ((ZVinrHqN7VE & (NL-1))) << 3);
      OOhuDz55Y78rH7   = ({DATA_W2{1'b1}}) >> (NL - ((ZVinrHqN7VE & (NL-1))) << 6);
      XUL              = j22r5RZ9Aec4l << (ZVinrHqN7VE & (NL-1));
      qSpj             = eqOrL6s2n << ((ZVinrHqN7VE & (NL-1)) << 6);
      k3I0Kwk7jw       = 0;
      rXb6Mt1Zpe2      = qWmdo6hK5;
      aywFyeD3         = 0;
      MAx = (ZVinrHqN7VE & (NL-1)) + W8pUYo0es;
      if(MAx > NL) begin
         k3I0Kwk7jw = 'b1;
         for(int m=0; m<NL; m++) begin
            aywFyeD3 = (ZVinrHqN7VE & (NL-1)) + m;
            rXb6Mt1Zpe2[(64*aywFyeD3)+:64] = eqOrL6s2n[(64*m)+:64];
         end
      end
      hHxvMimagtwiq65    = ({NL{|j22r5RZ9Aec4l}} & hEko7jtLxxmWEz)   | ({NL{X8zUELUV}} & HCS5CwLG);
      SKvxd0pCsRaiTpGr   = ({(Ma/8){|j22r5RZ9Aec4l}}     & Lnpi4RWnDP)   | ({(8*NL){X8zUELUV}} & KQPEQPLJDd3vbWp);
      XUL                = XUL                                           | ({NL{X8zUELUV}} & HCS5CwLG);
      qSpj               = qSpj                                          | ({(64*NL){X8zUELUV}} & qWmdo6hK5 & OOhuDz55Y78rH7);
      //Only send the credits which are safe to i.e. already written to staging (not held)
      //wptr_incr_safe = $countbits(vld, '1);
      a22aO48x4eVsBW             = 0;
      for(int m=0; m<NL; m++) begin
         a22aO48x4eVsBW += XUL[m];
      end
   end

   assign fYsZs52E4uCBc      = ZVinrHqN7VE      + W8pUYo0es;
   always_ff @(posedge clk_src) begin
      ZVinrHqN7VE       <= fYsZs52E4uCBc;  // sm_par

      if(NL == 1) begin
         X8zUELUV  <= '0; //sm_par
         qWmdo6hK5 <= '0;
      end
      else begin
         X8zUELUV   <= k3I0Kwk7jw;     // sm_par
         if (k3I0Kwk7jw) begin
            qWmdo6hK5  <= rXb6Mt1Zpe2;
         end
      end

      wen_n   <= ~XUL;  // sm_par
      addr_o  <= ZVinrHqN7VE[PTR_W-1:LOG_NL]; // sm_par

      w8JxCOz1   <= (!w8JxCOz1) ? &j22r5RZ9Aec4l : w8JxCOz1;

      if (|XUL) begin
         mask_o  <= ~{{2{hHxvMimagtwiq65[0]}}, SKvxd0pCsRaiTpGr}; // sm_par split (3x32) if(en_mwr_chk)
         data_o  <= qSpj[Ma-1:0]; // sm_par split (11x32) if(en_mwr_chk)
      end

      if(rst_src) begin
         wen_n    <= {NL{1'b1}}; // sm_par
         ZVinrHqN7VE        <= 'b0; // sm_par reset
         X8zUELUV           <= 'b0; // sm_par reset
         w8JxCOz1           <= '0;
      end
   end

   //CDC related code goes here

   assign sync_out.p0 = ptr_loc[0];
   assign sync_out.p1 = ptr_loc[1];
   assign sync_out.p2 = ptr_loc[2];
   assign sync_out.p3 = ptr_loc[3];

   assign ptr_fan[0]  = sync_in.p0;
   assign ptr_fan[1]  = sync_in.p1;
   assign ptr_fan[2]  = sync_in.p2;
   assign ptr_fan[3]  = sync_in.p3;

      credit_sync #(
      .NPTR(4),
      .INCR_W(INCR_W),
      .SYNC_W(10)
   ) u_credit_sync(
      .ZrhWtqi              (clk_src),
      .BmclJAsbT            (rst_src),
      .Q2HdxbEc             (a22aO48x4eVsBW),

      .OOhkfwuz             (clk_dst),
      .sM5EXEMYs8           (rst_dst),
      .WN1ymAOmS            (cred_incr),

      .iziBIr9Q8xkmfUdgn    (ptr_loc),
      .gXhJJZ4fHBUwp5EnWn   (ptr_fan)
   );

endmodule




`timescale 1ns/1ps

// ============================================================================
// 1. SELF-CHECKING PREAMBLE MACROS
// ============================================================================
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

// ============================================================================
// 2. MAIN SYSTEMVERILOG STRUCTURAL TESTBENCH
// ============================================================================
module tb_cdc_client;

  // Import the required package to access the packed 'quad_sync_t' struct
  import sync_pkg::*;

  `SVTEST_INIT

  // --------------------------------------------------------------------------
  // TOP-LEVEL ENVIRONMENT STRUCTURAL CONFIGURATIONS
  // --------------------------------------------------------------------------
  localparam IN_W = 256;
  localparam ADDR_W      = 8;
  localparam Ma                 = 64;
  localparam NL             = 2;
  localparam INCR_W         = 1;

  // Derived local parameters matching the internal implementation rules
  localparam DATA_W   = Ma;
  localparam MASK_W2   = (Ma/8);

  // --------------------------------------------------------------------------
  // INTERFACE STRUCT INTERACTION WIRING SIGNAL DECLARATIONS
  // --------------------------------------------------------------------------
  // Source / Client Clock Domain Logic Signals
  logic                            clk_src;
  logic                            rst_src;
  logic [IN_W-1:0]   in_flat;

  // Destination Domain Logic Signals
  logic                            clk_dst;
  logic                            rst_dst;

  // Captured Sub-system Outputs
  wire  [NL-1:0]               wen_n;
  wire  [ADDR_W-1:0]        addr_o;
  wire  [DATA_W-1:0]      data_o;
  wire  [MASK_W2-1:0]      mask_o;
  wire  [INCR_W-1:0]           cred_incr;
  wire  [3:0][9:0]                 ptr_fan;

  // Structural Handshake Macro Structures from test_pkg
  quad_sync_t                    sync_out;
  quad_sync_t                    sync_in;

  // --------------------------------------------------------------------------
  // DUAL-CLOCK ASYNCHRONOUS GENERATION LAYER
  // --------------------------------------------------------------------------
  // Client Clock Domain Driver (e.g., 200 MHz / 5.0ns Period)
  initial clk_src = 0;
  always #2.5 clk_src = ~clk_src;

  // Destination Clock Domain Driver (e.g., 133.33 MHz / 7.5ns Period)
  // Shifted completely out of phase to stress-test cross-domain transitions
  initial clk_dst = 0;
  always #3.75 clk_dst = ~clk_dst;

  // --------------------------------------------------------------------------
  // DESIGN UNDER TEST (DUT) INSTANTIATION
  // --------------------------------------------------------------------------
  cdc_client #(
    .IN_W (IN_W),
    .ADDR_W      (ADDR_W),
    .Ma                 (Ma),
    .NL             (NL),
    .INCR_W         (INCR_W)
  ) u_client (
    // Inputs
    .clk_src       (clk_src),
    .rst_src      (rst_src),
    .clk_dst             (clk_dst),
    .rst_dst            (rst_dst),
    .in_flat         (in_flat),
    .sync_in             (sync_in),

    // Outputs
    .wen_n       (wen_n),
    .addr_o      (addr_o),
    .data_o      (data_o),
    .mask_o      (mask_o),
    .sync_out           (sync_out),
    .cred_incr      (cred_incr),
    .ptr_fan  (ptr_fan)
  );

  // --------------------------------------------------------------------------
  // STRUCTURAL & COMPILER VALIDATION CHECKS
  // --------------------------------------------------------------------------
  initial begin
    // Synchronize and initialize port structures
    rst_src = 1;
    rst_dst       = 1;
    in_flat    = '0;

    // Wire loopback mapping to fulfill structural continuity
    sync_in.p0 = '0;
    sync_in.p1 = '0;
    sync_in.p2 = '0;
    sync_in.p3 = '0;

    #50;

    // Release resets asynchronously
    @(negedge clk_src);
    rst_src = 0;

    @(negedge clk_dst);
    rst_dst = 0;

    #20;

    $display("=====================================================");
    $display("Executing Structural Type & Boundary Validation Checks");
    $display("=====================================================");

    // Validation Check 1: Verify the top-level macro structure dimensions
    $display("  [STRUCT PATH] Checking sync_client package type token layout size...");
    `SVTEST_CHECK(($bits(sync_out) == 40), "STRUCT_BUG: Top-level quad_sync_t size mismatch! Expected 40 bits.")

    // Validation Check 2: Verify sub-element field bitwidths within the package structure
    `SVTEST_CHECK(($bits(sync_out.p0) == 10), "STRUCT_BUG: Sub-field sync ptr 0 size is not 10 bits.")
    `SVTEST_CHECK(($bits(sync_out.p3) == 10), "STRUCT_BUG: Sub-field sync ptr 3 size is not 10 bits.")

    // Validation Check 3: Check memory boundary connectivity alignments
    $display("  [STRUCT PATH] Checking layout parameters on internal SRAM vectors...");
    `SVTEST_CHECK(($bits(wen_n) == NL), "STRUCT_BUG: mem_wenB driver width mismatch.")
    `SVTEST_CHECK(($bits(addr_o) == ADDR_W), "STRUCT_BUG: mem_addrB driver width mismatch.")
    `SVTEST_CHECK(($bits(data_o) == Ma), "STRUCT_BUG: mem_dataB layout mismatch.")
    `SVTEST_CHECK(($bits(mask_o) == (Ma/8)), "STRUCT_BUG: mem_maskB sizing layout error.")

    // Validation Check 4: Verify multi-bit tracking ports match target width assignments
    `SVTEST_CHECK(($bits(ptr_fan) == 40), "STRUCT_BUG: Multi-pointer collection port sizing mismatch.")
    `SVTEST_CHECK(($bits(cred_incr) == INCR_W), "STRUCT_BUG: Credit increment signal width mismatch.")

    // Provide stimulus changes to verify structural responsiveness
    @(posedge clk_src);
    in_flat = {240'hDEADBEEF_CAFEF00D_12345678, 16'hFFFF}; // Drive active lanes
    sync_in.p0 = 10'h1A5;
    sync_in.p2 = 10'h25B;

    #100;

    // Check that top-level pins are transmitting properties down to lower layers
    `SVTEST_CHECK((ptr_fan[0] === 10'h1A5), "STRUCT_BUG: Loopback mapping value reflection failed.")
    `SVTEST_CHECK((ptr_fan[2] === 10'h25B), "STRUCT_BUG: Loopback mapping value reflection failed.")

    $display("=====================================================");
    $display("Executing Structural Type & Boundary Validation Checks");
    $display("=====================================================");

    `SVTEST_CHECK(($bits(sync_out) == 40), "STRUCT_BUG: Top-level quad_sync_t size mismatch! Expected 40 bits.")
    `SVTEST_CHECK(($bits(sync_out.p0) == 10), "STRUCT_BUG: Sub-field sync ptr 0 size is not 10 bits.")
    `SVTEST_CHECK(($bits(sync_out.p3) == 10), "STRUCT_BUG: Sub-field sync ptr 3 size is not 10 bits.")
    `SVTEST_CHECK(($bits(wen_n) == NL), "STRUCT_BUG: mem_wenB driver width mismatch.")
    `SVTEST_CHECK(($bits(addr_o) == ADDR_W), "STRUCT_BUG: mem_addrB driver width mismatch.")
    `SVTEST_CHECK(($bits(data_o) == Ma), "STRUCT_BUG: mem_dataB layout mismatch.")
    `SVTEST_CHECK(($bits(mask_o) == (Ma/8)), "STRUCT_BUG: mem_maskB sizing layout error.")
    `SVTEST_CHECK(($bits(ptr_fan) == 40), "STRUCT_BUG: Multi-pointer collection port sizing mismatch.")
    `SVTEST_CHECK(($bits(cred_incr) == INCR_W), "STRUCT_BUG: Credit increment signal width mismatch.")

    $display("=====================================================");
    $display("Structural Verification Phase Complete.");
    $display("=====================================================");

    // Parse macro pass/fail counts
    `SVTEST_PASSFAIL
    $finish;
  end

endmodule
