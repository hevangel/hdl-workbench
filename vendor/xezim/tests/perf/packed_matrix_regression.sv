`timescale 1ns/1ps

module packed_lane_pipe (
   input  logic                tick,
   input  logic                clear,
   input  logic [15:0][1:0]    rank_i,
   input  logic [15:0]         enable_i,
   input  logic [15:0][3:0]    tag_i,
   input  logic [15:0][31:0]   data_i,
   input  logic [15:0]         pick_i,
   input  logic [15:0]         allow_i,
   input  logic                carry_i,
   output logic                accept_o,
   output logic [3:0]          tag_o
);
   function automatic [15:0] decode_tag(input [3:0] tag);
      decode_tag = (1'b1 << tag);
   endfunction

   logic [15:0][1:0]  rank_q;
   logic [15:0][3:0]  tag_q;
   logic [15:0][15:0] decoded_q;
   logic [15:0]       enable_q;
   logic [15:0][31:0] data_q;

   logic [31:0] chosen_data;
   logic        chosen_enable;
   logic [3:0]  chosen_tag;
   logic [15:0] chosen_decoded;

   always @(posedge tick) begin
      if (clear) begin
         rank_q    <= '0;
         enable_q  <= '0;
         tag_q     <= '0;
         decoded_q <= '0;
         data_q    <= '0;
      end else begin
         for (int slot = 0; slot < 16; slot++) begin
            rank_q[slot]    <= rank_i[slot];
            enable_q[slot]  <= enable_i[slot];
            tag_q[slot]     <= tag_i[slot];
            decoded_q[slot] <= decode_tag(tag_i[slot]);
            data_q[slot]    <= data_i[slot];
         end
      end
   end

   always_comb begin
      accept_o = (|(chosen_decoded & allow_i)
                 | (carry_i & (chosen_decoded & {1'b1, allow_i[15:1]})))
                 & chosen_enable;
      tag_o = chosen_tag;
   end

   always_comb begin
      case (pick_i)
         16'h0001: begin
            chosen_data = data_q[0]; chosen_enable = enable_q[0];
            chosen_tag = tag_q[0]; chosen_decoded = decoded_q[0];
         end
         16'h0002: begin
            chosen_data = data_q[1]; chosen_enable = enable_q[1];
            chosen_tag = tag_q[1]; chosen_decoded = decoded_q[1];
         end
         16'h0004: begin
            chosen_data = data_q[2]; chosen_enable = enable_q[2];
            chosen_tag = tag_q[2]; chosen_decoded = decoded_q[2];
         end
         16'h0008: begin
            chosen_data = data_q[3]; chosen_enable = enable_q[3];
            chosen_tag = tag_q[3]; chosen_decoded = decoded_q[3];
         end
         16'h0010: begin
            chosen_data = data_q[4]; chosen_enable = enable_q[4];
            chosen_tag = tag_q[4]; chosen_decoded = decoded_q[4];
         end
         16'h0020: begin
            chosen_data = data_q[5]; chosen_enable = enable_q[5];
            chosen_tag = tag_q[5]; chosen_decoded = decoded_q[5];
         end
         16'h0040: begin
            chosen_data = data_q[6]; chosen_enable = enable_q[6];
            chosen_tag = tag_q[6]; chosen_decoded = decoded_q[6];
         end
         16'h0080: begin
            chosen_data = data_q[7]; chosen_enable = enable_q[7];
            chosen_tag = tag_q[7]; chosen_decoded = decoded_q[7];
         end
         16'h0100: begin
            chosen_data = data_q[8]; chosen_enable = enable_q[8];
            chosen_tag = tag_q[8]; chosen_decoded = decoded_q[8];
         end
         16'h0200: begin
            chosen_data = data_q[9]; chosen_enable = enable_q[9];
            chosen_tag = tag_q[9]; chosen_decoded = decoded_q[9];
         end
         16'h0400: begin
            chosen_data = data_q[10]; chosen_enable = enable_q[10];
            chosen_tag = tag_q[10]; chosen_decoded = decoded_q[10];
         end
         16'h0800: begin
            chosen_data = data_q[11]; chosen_enable = enable_q[11];
            chosen_tag = tag_q[11]; chosen_decoded = decoded_q[11];
         end
         16'h1000: begin
            chosen_data = data_q[12]; chosen_enable = enable_q[12];
            chosen_tag = tag_q[12]; chosen_decoded = decoded_q[12];
         end
         16'h2000: begin
            chosen_data = data_q[13]; chosen_enable = enable_q[13];
            chosen_tag = tag_q[13]; chosen_decoded = decoded_q[13];
         end
         16'h4000: begin
            chosen_data = data_q[14]; chosen_enable = enable_q[14];
            chosen_tag = tag_q[14]; chosen_decoded = decoded_q[14];
         end
         16'h8000: begin
            chosen_data = data_q[15]; chosen_enable = enable_q[15];
            chosen_tag = tag_q[15]; chosen_decoded = decoded_q[15];
         end
         default: begin
            chosen_data = data_q[0]; chosen_enable = 1'b0;
            chosen_tag = tag_q[0]; chosen_decoded = decoded_q[0];
         end
      endcase
   end
endmodule

module packed_lane_farm (
   input  logic                     tick,
   input  logic                     clear,
   input  logic [16:0][15:0][1:0]   farm_rank_i,
   input  logic [16:0][15:0]        farm_enable_i,
   input  logic [16:0][15:0][3:0]   farm_tag_i,
   input  logic [16:0][15:0][31:0]  farm_data_i,
   input  logic [16:0][15:0]        farm_pick_i,
   input  logic [16:0][15:0]        farm_allow_i,
   input  logic [16:0]              farm_carry_i,
   output logic [16:0]              farm_accept_o,
   output logic [16:0][3:0]         farm_tag_o
);
   genvar cell_id;
   generate
      for (cell_id = 0; cell_id < 17; cell_id++) begin : cell_gen
         packed_lane_pipe u_cell (
            .tick(tick), .clear(clear),
            .rank_i(farm_rank_i[cell_id]),
            .enable_i(farm_enable_i[cell_id]),
            .tag_i(farm_tag_i[cell_id]),
            .data_i(farm_data_i[cell_id]),
            .pick_i(farm_pick_i[cell_id]),
            .allow_i(farm_allow_i[cell_id]),
            .carry_i(farm_carry_i[cell_id]),
            .accept_o(farm_accept_o[cell_id]),
            .tag_o(farm_tag_o[cell_id])
         );
      end
   endgenerate
endmodule

module packed_matrix_regression_top;
   localparam int ITERATIONS = 256;
   localparam int CLEAR_END  = ITERATIONS * 1 / 100;
   localparam int X_END      = CLEAR_END + (ITERATIONS * 80 / 100);
   localparam int TOGGLE_END = X_END + (ITERATIONS * 8 / 100);

   logic tick;
   logic clear;

   logic [16:0][15:0][1:0]  bank_a_rank;
   logic [16:0][15:0]       bank_a_enable;
   logic [16:0][15:0][3:0]  bank_a_tag;
   logic [16:0][15:0][31:0] bank_a_data;
   logic [16:0][15:0]       bank_a_pick;
   logic [16:0][15:0]       bank_a_allow;
   logic [16:0]             bank_a_carry;
   wire  [16:0]             bank_a_accept;
   wire  [16:0][3:0]        bank_a_tag_o;

   logic [16:0][15:0][1:0]  bank_b_rank;
   logic [16:0][15:0]       bank_b_enable;
   logic [16:0][15:0][3:0]  bank_b_tag;
   logic [16:0][15:0][31:0] bank_b_data;
   logic [16:0][15:0]       bank_b_pick;
   logic [16:0][15:0]       bank_b_allow;
   logic [16:0]             bank_b_carry;
   wire  [16:0]             bank_b_accept;
   wire  [16:0][3:0]        bank_b_tag_o;

   packed_lane_farm u_bank_a (
      .tick(tick), .clear(clear),
      .farm_rank_i(bank_a_rank), .farm_enable_i(bank_a_enable),
      .farm_tag_i(bank_a_tag), .farm_data_i(bank_a_data),
      .farm_pick_i(bank_a_pick), .farm_allow_i(bank_a_allow),
      .farm_carry_i(bank_a_carry), .farm_accept_o(bank_a_accept),
      .farm_tag_o(bank_a_tag_o)
   );

   packed_lane_farm u_bank_b (
      .tick(tick), .clear(clear),
      .farm_rank_i(bank_b_rank), .farm_enable_i(bank_b_enable),
      .farm_tag_i(bank_b_tag), .farm_data_i(bank_b_data),
      .farm_pick_i(bank_b_pick), .farm_allow_i(bank_b_allow),
      .farm_carry_i(bank_b_carry), .farm_accept_o(bank_b_accept),
      .farm_tag_o(bank_b_tag_o)
   );

   initial begin
      tick = 0;
      forever #5 tick = ~tick;
   end

   int selected_cell = 0;
   int rotate_count = 0;
   int observed_edges = 0;
   logic [15:0] pattern_state = 16'hace1;

   always @(posedge tick) observed_edges++;

   function automatic logic [15:0] next_pattern(ref logic [15:0] state);
      state = (state >> 1) ^ (-(state & 1'b1) & 16'hb400);
      return state;
   endfunction

   initial begin
      string waveform_path;
      if ($value$plusargs("PACKED_MATRIX_VCD=%s", waveform_path)) begin
         $dumpfile(waveform_path);
         $dumpvars(1, packed_matrix_regression_top);
      end
   end

   initial begin
      clear         = 1'b1;
      bank_a_rank   = '0; bank_a_enable = '0; bank_a_tag   = '0;
      bank_a_data   = '0; bank_a_pick   = '0; bank_a_allow = '0;
      bank_a_carry  = '0;
      bank_b_rank   = '0; bank_b_enable = '0; bank_b_tag   = '0;
      bank_b_data   = '0; bank_b_pick   = '0; bank_b_allow = '0;
      bank_b_carry  = '0;

      for (int round = 0; round < ITERATIONS; round++) begin
         @(negedge tick);
         if (round < CLEAR_END) begin
            clear = 1'b1;
            for (int unit = 0; unit < 17; unit++) begin
               bank_a_enable[unit] = '0; bank_a_tag[unit] = '0;
               bank_b_enable[unit] = '0; bank_b_tag[unit] = '0;
            end
         end else if (round < X_END) begin
            clear = 1'b0;
            for (int unit = 0; unit < 17; unit++) begin
               for (int slot = 0; slot < 16; slot++) begin
                  bank_a_rank[unit][slot] = 2'bx0;
                  bank_b_rank[unit][slot] = 2'bx0;
               end
            end
         end else if (round < TOGGLE_END) begin
            for (int unit = 0; unit < 17; unit++) begin
               if (unit != selected_cell) begin
                  for (int slot = 0; slot < 16; slot++) begin
                     bank_a_rank[unit][slot] = (round % 2 == 0) ? 2'bx0 : 2'b0x;
                     bank_b_rank[unit][slot] = (round % 2 == 0) ? 2'bx0 : 2'b0x;
                  end
                  bank_a_enable[unit] = '0; bank_a_tag[unit] = '0;
                  bank_b_enable[unit] = '0; bank_b_tag[unit] = '0;
               end
            end

            for (int slot = 0; slot < 16; slot++) begin
               bank_a_rank[selected_cell][slot] = (round % 2 == 0) ? 2'b01 : 2'b10;
               bank_b_rank[selected_cell][slot] = (round % 2 == 0) ? 2'b01 : 2'b10;
            end

            if (round % 3 == 0) begin
               for (int slot = 0; slot < 16; slot++) begin
                  logic [15:0] bits;
                  bits = next_pattern(pattern_state);
                  bank_a_enable[selected_cell][slot] = bits[0];
                  bank_a_tag[selected_cell][slot] = bits[4:1];
                  bits = next_pattern(pattern_state);
                  bank_b_enable[selected_cell][slot] = bits[0];
                  bank_b_tag[selected_cell][slot] = bits[4:1];
               end
            end

            rotate_count++;
            if (rotate_count > 20) begin
               rotate_count = 0;
               selected_cell = (selected_cell + 1) % 17;
            end
         end else begin
            for (int unit = 0; unit < 17; unit++) begin
               bank_a_rank[unit] = '0; bank_a_enable[unit] = '0; bank_a_tag[unit] = '0;
               bank_b_rank[unit] = '0; bank_b_enable[unit] = '0; bank_b_tag[unit] = '0;
            end
         end

         for (int unit = 0; unit < 17; unit++) begin
            bank_a_pick[unit] = (1 << (round % 16));
            bank_b_pick[unit] = (1 << (round % 16));
            bank_a_allow[unit] = 16'hffff;
            bank_b_allow[unit] = 16'hffff;
         end
      end

      @(posedge tick);
      #1;
      if (observed_edges < ITERATIONS
          || bank_a_accept !== '0 || bank_b_accept !== '0
          || bank_a_tag_o !== '0 || bank_b_tag_o !== '0) begin
         $display("REGRESSION_FAIL edges=%0d", observed_edges);
         $fatal(2);
      end
      $display("REGRESSION_OK cycles=%0d edges=%0d", ITERATIONS, observed_edges);
      $finish;
   end
endmodule
