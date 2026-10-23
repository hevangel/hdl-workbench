package attrs_pkg;
   typedef struct packed {
      logic           f_hst;
      logic [9:0]     f_bcnt;
      logic [9:0]     t_bcnt;
      logic [4:0]     fb_bcnt;
      logic [3:0]     fb_64_cnt;
      logic [3:0]     pl_cnt;
      logic           l_bnk;
      logic           f_brq;
      logic [1:0]     rd_aln;
      logic           wrp;
      logic           wrp_ch_msk;
      logic [9:0]     wrp_f_bcnt;
      logic [9:0]     wrp_ofst;
   } lane_rd_attr_t;
   typedef struct packed {
      logic           f_hst;
      logic [11:0]    f_bcnt;
      logic [4:0]     fb_bcnt;
      logic [11:0]    t_bcnt;
      logic [3:0]     fb_64_cnt;
      logic [3:0]     pl_cnt;
      logic           l_bnk;
      logic           f_brq;
      logic [1:0]     rd_aln;
      logic           wrp_ch_msk;
      logic           wrp;
      logic [11:0]    wrp_f_bcnt;
      logic [11:0]    wrp_ofst;
   } lane_wr_attr_t;
endpackage: attrs_pkg

module flow_stg #(
   parameter DW = 1, parameter VW = 1, parameter BC = 1,
   parameter RS = 1, parameter XD = 1, parameter RD = 0
) (
   input  logic          i_clk,
   input  logic          i_rst,
   input  logic [VW-1:0] i_vld,
   output logic          o_nxt,
   input  logic [DW-1:0] i_dat,
   output logic [VW-1:0] o_vld,
   input  logic          i_nxt,
   output logic [DW-1:0] o_dat
);
   logic          s_ld;
   logic [VW-1:0] s_vld_rs;
   logic          s_nxt_rs;
   assign s_vld_rs = i_vld;
   assign o_nxt    = s_nxt_rs;
   assign s_nxt_rs = BC ? (i_nxt | !(|o_vld)) : i_nxt;
   assign s_ld     = (|s_vld_rs) & s_nxt_rs;
   always_ff @(posedge i_clk) begin
      o_vld <= (i_rst) ? 0 : s_nxt_rs ? s_vld_rs : o_vld;
      o_dat <= s_ld ? i_dat : o_dat;
      if (RD && i_rst) o_dat <= 0;
   end
endmodule

module skid_stg #(
   parameter DW = 1, parameter VW = 1, parameter BC = 1
) (
   input  logic          i_clk,
   input  logic          i_rst,
   input  logic [VW-1:0] i_vld,
   output logic          o_nxt,
   input  logic [DW-1:0] i_dat,
   output logic [VW-1:0] o_vld,
   input  logic          i_nxt,
   output logic [DW-1:0] o_dat
);
   logic [VW-1:0] s_buf_vld;
   logic [DW-1:0] s_buf_dat;
   logic          s_buf_sel;
   logic          s_ld;
   always @(posedge i_clk) begin
      if (i_rst) begin
         s_buf_vld <= 0;
         s_buf_sel <= 0;
      end else begin
         s_buf_vld <= i_nxt ? 0 : s_ld ? i_vld : s_buf_vld;
         s_buf_sel <= i_nxt ? 0 : s_ld ? 1'b1 : s_buf_sel;
      end
      s_buf_dat <= i_nxt ? s_buf_dat : s_ld ? i_dat : s_buf_dat;
   end
   always_comb begin
      o_nxt = !s_buf_sel;
      s_ld  = !s_buf_sel & (|i_vld) & !i_nxt;
      o_vld = s_buf_sel ? s_buf_vld : i_vld;
      o_dat = s_buf_sel ? s_buf_dat : i_dat;
   end
endmodule

module flow_skid #(
   parameter DW = 1, parameter VW = 1, parameter BC = 1,
   parameter RS = 1, parameter XD = 1, parameter RD = 0
) (
   input  logic          i_clk,
   input  logic          i_rst,
   input  logic [VW-1:0] i_vld,
   output logic          o_nxt,
   input  logic [DW-1:0] i_dat,
   output logic [VW-1:0] o_vld,
   input  logic          i_nxt,
   output logic [DW-1:0] o_dat
);
   logic [VW-1:0] s_mvld;
   logic          s_mnxt;
   logic [DW-1:0] s_mdat;
   flow_stg #(.DW(DW), .VW(VW), .BC(BC), .RS(RS), .XD(XD), .RD(RD)) u_pipe (
      .i_clk(i_clk), .i_rst(i_rst), .i_vld(i_vld), .o_nxt(o_nxt), .i_dat(i_dat),
      .o_vld(s_mvld), .i_nxt(s_mnxt), .o_dat(s_mdat)
   );
   skid_stg #(.DW(DW), .VW(VW), .BC(BC)) u_skid (
      .i_clk(i_clk), .i_rst(i_rst), .i_vld(s_mvld), .o_nxt(s_mnxt), .i_dat(s_mdat),
      .o_vld(o_vld), .i_nxt(i_nxt), .o_dat(o_dat)
   );
endmodule

module lane_expander import attrs_pkg::*; #(
   parameter P_CNLANES   = 4,
   parameter P_LOG_LANES = $clog2(P_CNLANES),
   parameter type attr_t = lane_rd_attr_t
)(
   input  logic                                  i_gclk,
   input  logic                                  i_srst,
   input  logic                                  i_c_fhst,
   input  logic                                  i_c_nhst_m1,
   input  logic [3:0]                            i_c_nbnk_m1,
   input  logic [2:0]                            i_c_lbst_sz,
   input  logic [12:0]                           i_c_itlv_sz,
   input  logic [P_CNLANES-1:0]                  i_ln_rd,
   input  logic                                  i_n_pat_vld,
   input  attr_t                                 i_n_pat,
   output logic                                  o_n_pat_rdy,
   output logic [1:0]                            o_n_pat_ps_rdy,
   output attr_t              [P_CNLANES:0]      o_pat,
   output logic               [P_CNLANES:0]      o_pat_s_req,
   output logic               [P_CNLANES:0]      o_pat_s_hst,
   output logic               [P_CNLANES:0]      o_pat_s_bnk,
   output logic               [P_CNLANES:0]      o_pat_s_chnk
);
   attr_t                                        s_pat_st;
   attr_t                                        s_nxt_pat;
   attr_t                                        s_nxt_pat_mx;
   attr_t                                        s_n_pat_ps;
   logic                                         s_n_pat_ps_vld;
   logic [P_CNLANES:0][P_CNLANES:1][12:0]        s_nxt_pat_t_bcnt_ac;
   logic [P_CNLANES:1][12:0]                     s_nxt_pat_t_bcnt;
   logic [P_CNLANES:1][12:0]                     s_pat_st_t_bcnt;
   logic [4:0]                                   s_lns_p_bnk;
   logic [P_CNLANES:1][4:0]                      s_lns_p_bnk_m_n;
   logic [P_CNLANES:1][9:0]                      s_lns_p_hst_m_n;
   logic [3:0]                                   s_f_hst_idx;
   logic [3:0]                                   s_f_hst_idx_p1;
   logic [3:0]                                   s_f_hst_idx_p1_mskd;
   logic [3:0]                                   s_f_hst_p1;
   logic [P_CNLANES:1][9:0]                      s_nxt_pat_f_bcnt_mn;
   logic [P_CNLANES:1][9:0]                      s_pat_st_f_bcnt_mn;
   logic [P_CNLANES:1][9:0]                      s_nxt_pat_f_bcnt_pc;
   logic [P_CNLANES:1][9:0]                      s_pat_st_f_bcnt_pc;
   logic [P_CNLANES:1][P_CNLANES:0]              s_nxt_pat_s_req;
   logic [P_CNLANES:1][P_CNLANES:0]              s_nxt_pat_s_hst;
   logic [P_CNLANES:1][P_CNLANES:0]              s_nxt_pat_s_bnk;
   logic [P_CNLANES:1][P_CNLANES:0]              s_nxt_pat_s_chnk;
   logic [P_CNLANES:1][P_CNLANES:0]              s_nxt_pat_s_plvl;
   logic [P_CNLANES:0]                           s_pat_s_plvl;
   logic [1:0]                                   s_wrp_st;
   logic [1:0]                                   s_wrp_st_lst;
   logic                                         s_itlv_typ;
   logic [3:0]                                   s_wrp_ch_msk;
   logic [1:0]                                   s_wrp_syn_bd;
   logic                                         s_c_fhst_m1;

   flow_skid #(.DW($bits(attr_t))) u_psos (
      .i_clk   (i_gclk),          .i_rst   (i_srst),
      .i_vld   (i_n_pat_vld),     .o_nxt   (o_n_pat_rdy),
      .i_dat   (i_n_pat),         .o_vld   (s_n_pat_ps_vld),
      .i_nxt   (|o_n_pat_ps_rdy), .o_dat   (s_n_pat_ps)
   );

   always_ff @(posedge i_gclk) begin
      s_lns_p_bnk <= (1 << i_c_lbst_sz) >> 3;
      for (int i = 1; i <= P_CNLANES; i++) begin
         s_lns_p_bnk_m_n[i] <= ((1 << i_c_lbst_sz) >> 3) - i;
         s_lns_p_hst_m_n[i] <= (i_c_itlv_sz >> 3) - i;
      end
      s_c_fhst_m1 <= i_c_fhst - 1;
      s_itlv_typ  <= (i_c_itlv_sz == 512 || i_c_itlv_sz == 1024 || i_c_itlv_sz == 2048) ? 1'b1 : 1'b0;
   end

   always_comb begin : calc_next_pattr
      s_nxt_pat = s_pat_st;
      for (int i = 1; i <= P_CNLANES; i++) begin
         if(i_ln_rd[i-1]) begin
            s_nxt_pat = o_pat[i];
         end
      end
      s_nxt_pat_mx = s_nxt_pat;
      if (s_nxt_pat_mx.t_bcnt == 0) begin
         s_nxt_pat = 0;
         if (s_n_pat_ps_vld) begin
            s_nxt_pat            = s_n_pat_ps;
            s_nxt_pat.wrp_f_bcnt = s_n_pat_ps.wrp_f_bcnt + 1;
            s_nxt_pat.f_bcnt     = s_n_pat_ps.f_bcnt + 1;
            s_nxt_pat.fb_bcnt    = s_n_pat_ps.fb_bcnt + 1;
            s_nxt_pat.l_bnk      = 0;
            s_nxt_pat.t_bcnt     = s_n_pat_ps.t_bcnt + 1;
            s_nxt_pat.fb_64_cnt  = (s_n_pat_ps.fb_bcnt > 7) ? s_n_pat_ps.f_brq ? (s_n_pat_ps.fb_bcnt - 7) : 8 : (s_n_pat_ps.fb_bcnt + 1);
            s_nxt_pat.pl_cnt     = s_n_pat_ps.wrp & !s_itlv_typ ? s_n_pat_ps.t_bcnt <= 8 ? s_n_pat_ps.t_bcnt :
                                   i_c_lbst_sz == 6 ? 8 : (s_n_pat_ps.wrp_ofst > 7) ? (s_n_pat_ps.wrp_ofst - 7) : (s_n_pat_ps.wrp_ofst + 1) : s_nxt_pat.fb_64_cnt;
         end
      end
      for (int i = 1; i < P_CNLANES + 1; i++) begin
         s_nxt_pat_t_bcnt[i] = s_nxt_pat_mx.t_bcnt - i;
      end
      if (s_nxt_pat_mx.t_bcnt == 0) begin
         s_nxt_pat_t_bcnt = 0;
         if (s_n_pat_ps_vld) begin
            for (int i = 1; i < P_CNLANES + 1; i++) begin
               s_nxt_pat_t_bcnt[i] = s_n_pat_ps.t_bcnt + 1 - i;
            end
         end
      end
      for (int i = 1; i <= P_CNLANES; i++) begin
         s_nxt_pat_f_bcnt_mn[i] = s_nxt_pat_mx.f_bcnt - i;
         s_nxt_pat_f_bcnt_pc[i] = s_nxt_pat_mx.f_bcnt + s_lns_p_bnk_m_n[i];
         if(s_pat_st_t_bcnt[i] == 0) begin
            s_nxt_pat_f_bcnt_mn[i] = 0;
            s_nxt_pat_f_bcnt_pc[i] = 0;
            if (s_n_pat_ps_vld) begin
               s_nxt_pat_f_bcnt_mn[i] = s_n_pat_ps.f_bcnt + 1 - i;
               s_nxt_pat_f_bcnt_pc[i] = s_n_pat_ps.f_bcnt + 1 + s_lns_p_bnk_m_n[i];
            end
         end
      end
      o_n_pat_ps_rdy = '0;
      if(s_n_pat_ps_vld & s_nxt_pat_mx.t_bcnt == 0) begin
         for (int i = 0; i < 2; i++)
           o_n_pat_ps_rdy[i] = (s_n_pat_ps.f_hst == i);
      end
      s_nxt_pat_s_req   = 0;
      s_nxt_pat_s_hst  = 0;
      s_nxt_pat_s_bnk  = 0;
      s_nxt_pat_s_chnk = 0;
      s_nxt_pat_s_plvl = 0;
      for (int i = 1; i <= P_CNLANES; i++) begin
        s_nxt_pat_s_req[i][0]   = o_pat[i].t_bcnt > 0;
        s_nxt_pat_s_hst[i][0]  = 1;
        s_nxt_pat_s_bnk[i][0]  = 1;
        s_nxt_pat_s_chnk[i][0] = 1;
        s_nxt_pat_s_plvl[i][0] = 1;
        for (int j = 1; j <= P_CNLANES; j++) begin
           s_nxt_pat_s_req[i][j]   = (o_pat[i].t_bcnt > j);
           s_nxt_pat_s_hst[i][j]  = (o_pat[i].f_bcnt > j);
           s_nxt_pat_s_bnk[i][j]  = (o_pat[i].fb_bcnt > j);
           s_nxt_pat_s_chnk[i][j] = (o_pat[i].fb_64_cnt > j);
           s_nxt_pat_s_plvl[i][j] = (o_pat[i].pl_cnt > j);
        end
     end
   end

always_ff @(posedge i_gclk) begin : flop_pattr_sts
   s_pat_st <= s_nxt_pat;
   if (s_n_pat_ps_vld & (|o_n_pat_ps_rdy)) begin
      s_wrp_ch_msk <= ((s_nxt_pat.t_bcnt == 32) & (i_c_itlv_sz == 64)) ? 4'b1100 :
                      ((s_nxt_pat.t_bcnt == 16) | ((i_c_itlv_sz == 128) & (s_nxt_pat.t_bcnt == 32))) ? 4'b1110 : 4'b1111;
      s_wrp_syn_bd <= (i_c_itlv_sz == 64) ? (s_nxt_pat.t_bcnt >> 3) - ((s_nxt_pat.f_hst - i_c_fhst) & (2'b11 >> (s_nxt_pat.t_bcnt == 16))) - 1'b1 :
                      (s_nxt_pat.t_bcnt >> 4) - ((s_nxt_pat.f_hst - i_c_fhst) & 1'b1) - 1'b1;
   end
   s_pat_st_t_bcnt    <= s_nxt_pat_t_bcnt;
   s_pat_st_f_bcnt_mn <= s_nxt_pat_f_bcnt_mn;
   s_pat_st_f_bcnt_pc <= s_nxt_pat_f_bcnt_pc;
   if ((o_pat[0].wrp_f_bcnt == 2) & (i_ln_rd > 0)) begin
      s_wrp_st <= s_wrp_st + 1;
   end
   for (int i = 1; i <= P_CNLANES; i++) begin
      if (i_ln_rd[i-1]) begin
         o_pat_s_req   <= s_nxt_pat_s_req[i];
         o_pat_s_hst   <= s_nxt_pat_s_hst[i];
         o_pat_s_bnk   <= s_nxt_pat_s_bnk[i];
         o_pat_s_chnk  <= s_nxt_pat_s_chnk[i];
         s_pat_s_plvl  <= s_nxt_pat_s_plvl[i];
      end
   end
   if (s_nxt_pat_mx.t_bcnt == 0) begin
      o_pat_s_req    <= 0;
      o_pat_s_hst   <= 0;
      o_pat_s_bnk   <= 0;
      o_pat_s_chnk  <= 0;
      s_pat_s_plvl   <= 0;
      s_wrp_st        <= 0;
      s_wrp_st_lst   <= 1;
      if (s_n_pat_ps_vld) begin
         o_pat_s_req[0]   <= 1;
         o_pat_s_hst[0]  <= 1;
         o_pat_s_bnk[0]  <= 1;
         o_pat_s_chnk[0] <= 1;
         s_pat_s_plvl[0]  <= 1;
         for (int j = 1; j <= P_CNLANES; j++) begin
            o_pat_s_req[j]   <= (s_nxt_pat.t_bcnt > j);
            o_pat_s_hst[j]  <= (s_nxt_pat.f_bcnt > j);
            o_pat_s_bnk[j]  <= (s_nxt_pat.fb_bcnt > j);
            o_pat_s_chnk[j] <= (s_nxt_pat.fb_64_cnt > j);
            s_pat_s_plvl[j]  <= (s_nxt_pat.pl_cnt > j);
         end
         s_wrp_st_lst  <= ((s_nxt_pat.t_bcnt-1) >> (i_c_lbst_sz-3)) | 1'b1;
      end
   end
   if (i_srst) begin
      s_pat_st            <= 'd0;
      s_pat_st_t_bcnt     <= 'd0;
      s_pat_st_f_bcnt_mn  <= 'd0;
      s_pat_st_f_bcnt_pc  <= 'd0;
      o_pat_s_req         <= 0;
      o_pat_s_hst        <= 0;
      o_pat_s_bnk        <= 0;
      o_pat_s_chnk       <= 0;
      s_pat_s_plvl        <= 0;
      s_wrp_st             <= 0;
      s_wrp_st_lst        <= 1;
      s_wrp_ch_msk         <= 0;
      s_wrp_syn_bd         <= 0;
   end
end

assign s_f_hst_idx         = o_pat[0].f_hst - i_c_fhst;
assign s_f_hst_idx_p1      = o_pat[0].f_hst - s_c_fhst_m1;
assign s_f_hst_idx_p1_mskd = (o_pat[0].wrp & !s_itlv_typ & (s_wrp_st == s_wrp_syn_bd)) ? (s_f_hst_idx & s_wrp_ch_msk) : (s_f_hst_idx_p1 & i_c_nhst_m1);
assign s_f_hst_p1          = s_f_hst_idx_p1_mskd + i_c_fhst;

always_comb begin
   o_pat = {(P_CNLANES+1){s_pat_st}};
   for (int j = 1; j <= P_CNLANES; j++) begin
      o_pat[j].t_bcnt     = 0;
      o_pat[j].f_bcnt     = 0;
      o_pat[j].wrp_f_bcnt = 0;
      if (o_pat_s_req[j]) begin
         o_pat[j].wrp_f_bcnt = (s_wrp_st == s_wrp_st_lst) ? (o_pat[0].t_bcnt - j) : i_c_itlv_sz[12:3];
         if (o_pat[0].wrp_f_bcnt > j) begin
            o_pat[j].wrp_f_bcnt = o_pat[0].wrp_f_bcnt - j;
         end
      end
      if (o_pat_s_req[j]) begin
         o_pat[j].t_bcnt = s_pat_st_t_bcnt[j];
         o_pat[j].f_bcnt = ((s_pat_st_t_bcnt[j] > i_c_itlv_sz[12:3]) & o_pat[0].wrp & !s_itlv_typ) ? i_c_itlv_sz[12:3] :
                           (s_pat_st_t_bcnt[j] > i_c_itlv_sz[12:3]) ? (o_pat[0].f_bcnt + i_c_itlv_sz[12:3] - j) : o_pat[j].t_bcnt;
         if (o_pat_s_hst[j] & ((o_pat[0].wrp_f_bcnt > j) | (!o_pat[0].wrp) | s_itlv_typ)) begin
            o_pat[j].f_bcnt = o_pat[0].f_bcnt - j;
         end
      end
      o_pat[j].rd_aln = 0;
      if (!o_pat[0].l_bnk & ~o_pat_s_bnk[j] & (o_pat[j].t_bcnt == o_pat[j].fb_bcnt)) begin
         o_pat[j].l_bnk = 1;
      end
      o_pat[j].f_hst = s_f_hst_p1;
      if (o_pat_s_hst[j] & ((o_pat[0].wrp_f_bcnt > j) | (!o_pat[0].wrp) | s_itlv_typ)) begin
         o_pat[j].f_hst = o_pat[0].f_hst;
      end
      o_pat[j].fb_bcnt = (o_pat[0].wrp & !s_itlv_typ & (o_pat[j].f_bcnt > s_lns_p_bnk)) ? s_lns_p_bnk :
                         ((o_pat[j].f_bcnt > s_lns_p_bnk) ? (o_pat[0].fb_bcnt + s_lns_p_bnk_m_n[j]) : o_pat[j].f_bcnt);
      o_pat[j].fb_64_cnt = (o_pat[0].wrp & !s_itlv_typ) ? ((s_wrp_st == s_wrp_st_lst) ? (o_pat[j].t_bcnt > 8 ? o_pat[j].t_bcnt - 8 : o_pat[j].t_bcnt) : 8) :
                           (o_pat[0].fb_bcnt + 8 - j);
      o_pat[j].pl_cnt    = (o_pat[0].wrp & !s_itlv_typ) ? ((s_wrp_st == s_wrp_st_lst) ? o_pat[j].t_bcnt : 8) :
                           (o_pat[0].pl_cnt + 8 - j);
      if (o_pat_s_bnk[j] & ((o_pat[0].wrp_f_bcnt > j) | (!o_pat[0].wrp) | s_itlv_typ)) begin
         o_pat[j].fb_bcnt   = o_pat[0].fb_bcnt - j;
         o_pat[j].fb_64_cnt = o_pat[0].fb_64_cnt + 8 - j;
         o_pat[j].pl_cnt    = (o_pat[0].wrp & !s_itlv_typ) ? (o_pat[0].f_bcnt - j) : (o_pat[0].pl_cnt + 8 - j);
         if (o_pat_s_chnk[j] & ((o_pat[0].wrp_f_bcnt > j) | (!o_pat[0].wrp) | s_itlv_typ)) begin
            o_pat[j].fb_64_cnt = o_pat[0].fb_64_cnt - j;
         end
         if (s_pat_s_plvl[j] & ((o_pat[0].wrp_f_bcnt > j) | (!o_pat[0].wrp) | s_itlv_typ)) begin
            o_pat[j].pl_cnt = o_pat[0].pl_cnt - j;
         end
      end
   end
end
endmodule

`define SVCHECK(condition) \
   if (!(condition)) begin \
      $display("SVCHECK FAILED at line %0d: %s", `__LINE__, `"condition`"); \
      tests_failed++; \
   end

`define SVTEST_PASSFAIL \
   if (tests_failed == 0) begin \
      $display(">>> ALL TESTS PASSED SUCCESSFULLY <<<"); \
   end else begin \
      $display("<<< %0d TEST(S) FAILED >>>", tests_failed); \
   end \
   $finish;

module tb_lane_exp;
   import attrs_pkg::*;
   int tests_failed = 0;
   logic clk;
   logic rst;
   logic        c_fhst;
   logic        c_nhst_m1;
   logic [3:0]  c_nbnk_m1;
   logic [2:0]  c_lbst_sz;
   logic [12:0] c_itlv_sz;
   logic [3:0]  ln_rd;
   logic        n_pat_vld;
   lane_rd_attr_t i_n_pat;
   logic        o_n_pat_rdy;
   logic [1:0]  o_n_pat_ps_rdy;
   lane_rd_attr_t [4:0] o_pat;
   logic [4:0]  pat_s_req;
   logic [4:0]  pat_s_hst;
   logic [4:0]  pat_s_bnk;
   logic [4:0]  pat_s_chnk;

   lane_expander #(
      .P_CNLANES(4),
      .attr_t(lane_rd_attr_t)
   ) uut (
      .i_gclk(clk),
      .i_srst(rst),
      .i_c_fhst(c_fhst),
      .i_c_nhst_m1(c_nhst_m1),
      .i_c_nbnk_m1(c_nbnk_m1),
      .i_c_lbst_sz(c_lbst_sz),
      .i_c_itlv_sz(c_itlv_sz),
      .i_ln_rd(ln_rd),
      .i_n_pat_vld(n_pat_vld),
      .i_n_pat(i_n_pat),
      .o_n_pat_rdy(o_n_pat_rdy),
      .o_n_pat_ps_rdy(o_n_pat_ps_rdy),
      .o_pat(o_pat),
      .o_pat_s_req(pat_s_req),
      .o_pat_s_hst(pat_s_hst),
      .o_pat_s_bnk(pat_s_bnk),
      .o_pat_s_chnk(pat_s_chnk)
   );

   logic [15:0] lfsr_reg;
   function automatic logic [15:0] step_lfsr(logic [15:0] current_val);
      logic lsb;
      lsb = current_val;
      step_lfsr = current_val >> 1;
      if (lsb) begin
         step_lfsr = step_lfsr ^ 16'hB400;
      end
   endfunction

   initial begin
      clk = 0;
      forever #5 clk = ~clk;
   end

   initial begin
      rst = 1;
      lfsr_reg = 16'hACE1;
      c_fhst    = 0;
      c_nhst_m1 = 0;
      c_nbnk_m1 = 4'd0;
      c_lbst_sz = 3'd3;
      c_itlv_sz = 13'd512;
      ln_rd     = 4'b0000;
      n_pat_vld = 0;
      i_n_pat   = '0;
      repeat(3) @(posedge clk);
      #1 rst = 0;
      $display("Executing Test Case 1: Initial State Reset Trace Validation");
      `SVCHECK(o_n_pat_rdy == 1'b1);
      `SVCHECK(pat_s_req == 5'b00000);
      $display("Executing Test Case 2: Deterministic Pseudo-Random LFSR Execution Loop");
      for (int cycle = 0; cycle < 50; cycle++) begin
         lfsr_reg = step_lfsr(lfsr_reg);
         c_fhst    = lfsr_reg;
         c_nhst_m1 = lfsr_reg;
         c_nbnk_m1 = lfsr_reg[5:2];
         c_lbst_sz = lfsr_reg[8:6];
         if (lfsr_reg[10:9] == 2'b00)      c_itlv_sz = 13'd512;
         else if (lfsr_reg[10:9] == 2'b01) c_itlv_sz = 13'd1024;
         else                              c_itlv_sz = 13'd2048;
         ln_rd     = lfsr_reg[14:11];
         n_pat_vld = lfsr_reg;
         i_n_pat.f_hst      = lfsr_reg;
         i_n_pat.f_bcnt     = {6'b000000, lfsr_reg[4:1]};
         i_n_pat.t_bcnt     = {6'b000000, lfsr_reg[8:5]};
         i_n_pat.fb_bcnt    = lfsr_reg[13:9];
         i_n_pat.fb_64_cnt  = lfsr_reg[3:0];
         i_n_pat.pl_cnt     = lfsr_reg[7:4];
         i_n_pat.wrp        = lfsr_reg;
         @(posedge clk);
         #1;
         `SVCHECK(!$isunknown(pat_s_req));
         `SVCHECK(!$isunknown(o_pat[0].t_bcnt));
         $display("Cycle %2d [LFSR:%4h] -> InVld:%b, OutRdy:%b, PatMsk:%b | Pat0_Tbcnt:%d",
                  cycle, lfsr_reg, n_pat_vld, o_n_pat_rdy, pat_s_req, o_pat[0].t_bcnt);
      end
      $display("Executing Test Case 3: Complete Functional Shutdown Evaluation");
      `SVTEST_PASSFAIL
   end
endmodule
