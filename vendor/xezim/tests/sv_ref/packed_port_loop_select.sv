// A lane-decode loop reads a packed multi-D INPUT port with a variable
// index:
//
//    for (int j = 0; j < 2; j++)
//       slot_dec[j] = (sel_vld[j] & sel_grp[j]) ? sel_dec[j] : 0;
//
// With sel_vld=2'b11, sel_grp=2'b11, sel_dec=8'h21 the two lanes decode
// slots 0 and 1, so slot_en must be 4'h3.
//
// If `sel_dec[j]` degrades to a one-BIT select the loop reads bit 0 (=1)
// and bit 1 (=0) instead of the nibbles 4'h1 and 4'h2, leaving
// slot_dec=8'h01 and slot_en=4'h1 -- lane 1's slot is silently dropped
// while any_en still asserts, so the failure is invisible at any_en and
// only surfaces as missing data downstream.
//
// Both instances below are the same module; they differ only in how the
// parent declares the net feeding sel_dec.

`timescale 1ns/1ps

module lane_decoder #(parameter int NSLOT = 4) (
   input  logic [1:0]             sel_vld,
   input  logic [1:0]             sel_grp,
   input  logic [1:0] [NSLOT-1:0] sel_dec,
   output logic       [NSLOT-1:0] slot_en,
   output logic                   any_en,
   output logic [1:0] [NSLOT-1:0] slot_dec_o,
   output int                     bits_sel_elem
);
   logic [1:0] [NSLOT-1:0] slot_dec;

   assign slot_dec_o    = slot_dec;
   assign bits_sel_elem = $bits(sel_dec[0]);

   always_comb begin
      slot_en = 'd0;
      any_en  = 1'b0;
      for (int j = 0; j < 2; j++) begin
         slot_dec[j] = (sel_vld[j] & sel_grp[j]) ? sel_dec[j] : 0;
         slot_en |= slot_dec[j];
      end
      any_en = |slot_en;
   end
endmodule

module packed_port_loop_select;

   logic [1:0]      sel_vld;
   logic [1:0]      sel_grp;

   logic [7:0]      dec_flat;         // parent net is a FLAT vector
   logic [1:0][3:0] dec_shaped;       // parent net matches the port shape

   logic [3:0]      en_flat,   en_shaped;
   logic            any_flat,  any_shaped;
   logic [1:0][3:0] dcd_flat,  dcd_shaped;
   int              bits_flat, bits_shaped;

   lane_decoder #(.NSLOT(4)) u_dec_flat (
      .sel_vld(sel_vld), .sel_grp(sel_grp),
      .sel_dec(dec_flat),
      .slot_en(en_flat), .any_en(any_flat),
      .slot_dec_o(dcd_flat), .bits_sel_elem(bits_flat));

   lane_decoder #(.NSLOT(4)) u_dec_shaped (
      .sel_vld(sel_vld), .sel_grp(sel_grp),
      .sel_dec(dec_shaped),
      .slot_en(en_shaped), .any_en(any_shaped),
      .slot_dec_o(dcd_shaped), .bits_sel_elem(bits_shaped));

   int n_checks = 0;
   int n_errors = 0;

   task automatic chk(string what, int got, int exp);
      n_checks++;
      if (got !== exp) begin
         n_errors++;
         $display("  FAIL  %-30s got=%0h  exp=%0h", what, got, exp);
      end
   endtask

   initial begin
      sel_vld = 2'b00; sel_grp = 2'b00;
      dec_flat = 8'h00; dec_shaped = 8'h00;
      #10;
      // Both lanes valid, both in group, lane0 -> slot0, lane1 -> slot1.
      sel_vld = 2'b11; sel_grp = 2'b11;
      dec_flat = 8'h21; dec_shaped = 8'h21;
      #10;

      $display("TEST packed_port_loop_select");
      $display("  flat-driven   : $bits(sel_dec[0])=%0d slot_dec=%0h slot_en=%0h any_en=%0b",
               bits_flat, dcd_flat, en_flat, any_flat);
      $display("  shaped-driven : $bits(sel_dec[0])=%0d slot_dec=%0h slot_en=%0h any_en=%0b",
               bits_shaped, dcd_shaped, en_shaped, any_shaped);

      chk("flat: $bits(sel_dec[0])", bits_flat,       4);
      chk("flat: slot_dec",          int'(dcd_flat),  8'h21);
      chk("flat: slot_en",           int'(en_flat),   4'h3);
      chk("flat: any_en",            int'(any_flat),  1'b1);

      chk("shaped: $bits(sel_dec[0])", bits_shaped,      4);
      chk("shaped: slot_dec",          int'(dcd_shaped), 8'h21);
      chk("shaped: slot_en",           int'(en_shaped),  4'h3);
      chk("shaped: any_en",            int'(any_shaped), 1'b1);

      $display("TEST packed_port_loop_select: %0d checks, %0d errors -> %s",
               n_checks, n_errors, (n_errors == 0) ? "PASS" : "FAIL");
      $finish;
   end
endmodule
