`timescale 1ns/1ps

typedef struct packed {
   logic [7:0] stamp;
   logic [2:0] region;
   logic       marker;
   logic [1:0] mode;
} cell_record_t;

typedef struct packed {
   logic [1:0][31:0] samples;
   cell_record_t [1:0] descriptors;
} bundle_t;

module slice_capture (
   input  logic                heartbeat,
   input  logic                clear,
   input  logic [1:0]          open_gate,
   input  bundle_t             source_bundle,
   output logic [1:0][31:0]    captured_sample,
   output logic [1:0][2:0]     captured_region,
   output logic [1:0][5:0]     captured_stamp,
   output logic [1:0]          captured_marker,
   output logic [1:0][1:0]     captured_mode
);
   always @(posedge heartbeat) begin
      if (clear) begin
         captured_sample <= '0;
         captured_region <= '0;
         captured_stamp  <= '0;
         captured_marker <= '0;
         captured_mode   <= '0;
      end else begin
         for (int position = 0; position < 2; position++) begin
            if (open_gate[position]) begin
               captured_sample[position] <= source_bundle.samples[position];
               captured_region[position] <= source_bundle.descriptors[position].region;
               captured_stamp[position]  <= source_bundle.descriptors[position].stamp[6:1];
               captured_marker[position] <= source_bundle.descriptors[position].marker;
               captured_mode[position]   <= source_bundle.descriptors[position].mode;
            end
         end
      end
   end
endmodule

module paired_capture (
   input  logic                 heartbeat,
   input  logic                 clear,
   input  logic [1:0][1:0]      open_grid,
   input  bundle_t [1:0]        parcel_grid,
   output logic [1:0][1:0][31:0] result_sample,
   output logic [1:0][1:0][2:0]  result_region,
   output logic [1:0][1:0][5:0]  result_stamp,
   output logic [1:0][1:0]       result_marker,
   output logic [1:0][1:0][1:0]  result_mode
);
   genvar branch;
   generate
      for (branch = 0; branch < 2; branch++) begin : capture_pair
         slice_capture unit (
            .heartbeat(heartbeat),
            .clear(clear),
            .open_gate(open_grid[branch]),
            .source_bundle(parcel_grid[branch]),
            .captured_sample(result_sample[branch]),
            .captured_region(result_region[branch]),
            .captured_stamp(result_stamp[branch]),
            .captured_marker(result_marker[branch]),
            .captured_mode(result_mode[branch])
         );
      end
   endgenerate
endmodule

module record_edge_check;
   logic heartbeat = 0;
   logic clear = 1;
   logic [1:0][1:0] open_grid = '0;
   bundle_t [1:0] parcel_grid;
   wire [1:0][1:0][31:0] result_sample;
   wire [1:0][1:0][2:0] result_region;
   wire [1:0][1:0][5:0] result_stamp;
   wire [1:0][1:0] result_marker;
   wire [1:0][1:0][1:0] result_mode;

   paired_capture fixture (
      .heartbeat(heartbeat),
      .clear(clear),
      .open_grid(open_grid),
      .parcel_grid(parcel_grid),
      .result_sample(result_sample),
      .result_region(result_region),
      .result_stamp(result_stamp),
      .result_marker(result_marker),
      .result_mode(result_mode)
   );

   always #5 heartbeat = ~heartbeat;

   initial begin
      int pace_sum = 0;
      parcel_grid = '0;
      for (int pace = 0; pace < 3; pace++) begin
         @(negedge heartbeat);
         pace_sum = pace_sum + pace + 1;
      end
      if (pace_sum != 6)
         $fatal(1, "paced loop mismatch");
      clear = 0;

      parcel_grid[0] = {
         32'h5566_7788, 32'h1122_3344,
         8'h3c, 3'h2, 1'b0, 2'h1,
         8'h96, 3'h5, 1'b1, 2'h2
      };
      parcel_grid[1] = {
         32'hddee_ff00, 32'h99aa_bbcc,
         8'h58, 3'h1, 1'b1, 2'h0,
         8'he2, 3'h7, 1'b0, 2'h3
      };

      open_grid = '1;
      @(posedge heartbeat);
      #1;

      if (result_sample[0][0] !== 32'h1122_3344 ||
          result_sample[0][1] !== 32'h5566_7788 ||
          result_sample[1][0] !== 32'h99aa_bbcc ||
          result_sample[1][1] !== 32'hddee_ff00 ||
          result_region[0][0] !== 3'h5 || result_region[0][1] !== 3'h2 ||
          result_region[1][0] !== 3'h7 || result_region[1][1] !== 3'h1 ||
          result_stamp[0][0] !== 6'h0b || result_stamp[0][1] !== 6'h1e ||
          result_stamp[1][0] !== 6'h31 || result_stamp[1][1] !== 6'h2c ||
          result_marker !== 4'b1001 ||
          result_mode[0][0] !== 2'h2 || result_mode[0][1] !== 2'h1 ||
          result_mode[1][0] !== 2'h3 || result_mode[1][1] !== 2'h0) begin
         $fatal(1, "packed record mismatch");
      end

      repeat (3) @(negedge heartbeat);
      parcel_grid[1] = {
         32'h0123_4567, 32'h89ab_cdef,
         8'ha4, 3'h6, 1'b0, 2'h2,
         8'h6a, 3'h3, 1'b1, 2'h1
      };
      @(posedge heartbeat);
      #1;
      if (result_sample[1][0] !== 32'h89ab_cdef ||
          result_sample[1][1] !== 32'h0123_4567 ||
          result_region[1][0] !== 3'h3 || result_region[1][1] !== 3'h6 ||
          result_stamp[1][0] !== 6'h35 || result_stamp[1][1] !== 6'h12 ||
          result_marker[1] !== 2'b01 ||
          result_mode[1][0] !== 2'h1 || result_mode[1][1] !== 2'h2) begin
         $fatal(1, "packed record rearm mismatch");
      end

      $display("PACKED_RECORD_OK");
      $finish;
   end
endmodule
