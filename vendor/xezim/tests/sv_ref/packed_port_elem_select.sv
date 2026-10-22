// IEEE 1800-2023 §7.4.1 / §23.3.3
//
// The element width of a packed multi-dimensional PORT is a property of the
// port's own declared type (the FORMAL). It must not be inherited from the
// expression connected to it (the ACTUAL).
//
// `input logic [1:0][3:0] pa` has two 4-bit elements, so `pa[1]` selects
// bits [7:4] whatever drives the port -- a flat vector, a concatenation, a
// part-select or a differently-shaped packed array.
//
// Reference: run on a golden simulator; every case must report `pass`.

`timescale 1ns/1ps

module elem_probe (
   input  logic [1:0][3:0] pa,
   output int              o_bits_pa,
   output int              o_bits_pa0,
   output logic [3:0]      o_pa0,
   output logic [3:0]      o_pa1
);
   assign o_bits_pa  = $bits(pa);
   assign o_bits_pa0 = $bits(pa[0]);
   assign o_pa0      = pa[0];
   assign o_pa1      = pa[1];
endmodule

module packed_port_elem_select;

   // ---- drivers, all carrying the bit pattern 8'b0010_0001 = 8'h21 -------
   logic [1:0][3:0]      d_shaped;          // matches the formal exactly
   logic [7:0]           d_flat;            // flat packed vector
   wire  [7:0]           d_wire;            // flat net
   logic [3:0]           d_hi, d_lo;        // for a concatenation
   logic [15:0]          d_wide;            // for a part-select
   logic [3:0][1:0]      d_othershape;      // same 8 bits, different packing
   logic [3:0][1:0][3:0] d_big;             // slice of a bigger packed array
   logic [1:0][3:0]      d_unpk [0:3];      // element of an unpacked array

   assign d_wire = d_flat;

   // ---- one probe instance per driver shape -----------------------------
   int         b_pa   [0:8];
   int         b_pa0  [0:8];
   logic [3:0] v_pa0  [0:8];
   logic [3:0] v_pa1  [0:8];

   elem_probe u0 (.pa(d_shaped),      .o_bits_pa(b_pa[0]), .o_bits_pa0(b_pa0[0]), .o_pa0(v_pa0[0]), .o_pa1(v_pa1[0]));
   elem_probe u1 (.pa(d_flat),        .o_bits_pa(b_pa[1]), .o_bits_pa0(b_pa0[1]), .o_pa0(v_pa0[1]), .o_pa1(v_pa1[1]));
   elem_probe u2 (.pa(d_wire),        .o_bits_pa(b_pa[2]), .o_bits_pa0(b_pa0[2]), .o_pa0(v_pa0[2]), .o_pa1(v_pa1[2]));
   elem_probe u3 (.pa({d_hi, d_lo}),  .o_bits_pa(b_pa[3]), .o_bits_pa0(b_pa0[3]), .o_pa0(v_pa0[3]), .o_pa1(v_pa1[3]));
   elem_probe u4 (.pa(d_wide[7:0]),   .o_bits_pa(b_pa[4]), .o_bits_pa0(b_pa0[4]), .o_pa0(v_pa0[4]), .o_pa1(v_pa1[4]));
   elem_probe u5 (.pa(d_othershape),  .o_bits_pa(b_pa[5]), .o_bits_pa0(b_pa0[5]), .o_pa0(v_pa0[5]), .o_pa1(v_pa1[5]));
   elem_probe u6 (.pa(d_big[2]),      .o_bits_pa(b_pa[6]), .o_bits_pa0(b_pa0[6]), .o_pa0(v_pa0[6]), .o_pa1(v_pa1[6]));
   elem_probe u7 (.pa(d_unpk[1]),     .o_bits_pa(b_pa[7]), .o_bits_pa0(b_pa0[7]), .o_pa0(v_pa0[7]), .o_pa1(v_pa1[7]));

   string names [0:7] = '{
      "shaped [1:0][3:0]", "flat logic [7:0]",  "flat wire [7:0]",
      "concat {hi,lo}",    "part-select [7:0]", "other shape [3:0][1:0]",
      "slice big[2]",      "unpacked elem arr[1]"
   };

   int n_checks = 0;
   int n_errors = 0;

   task automatic chk(string what, int got, int exp);
      n_checks++;
      if (got !== exp) begin
         n_errors++;
         $display("  FAIL  %-34s got=%0d  exp=%0d", what, got, exp);
      end
   endtask

   initial begin
      d_shaped     = 8'h21;
      d_flat       = 8'h21;
      d_hi         = 4'h2;
      d_lo         = 4'h1;
      d_wide       = 16'hBA21;
      d_othershape = 8'h21;
      d_big[2]     = 8'h21;
      d_unpk[1]    = 8'h21;
      #1;

      $display("TEST packed_port_elem_select");
      for (int i = 0; i < 8; i++) begin
         $display("  [%0d] %-24s $bits(pa)=%0d $bits(pa[0])=%0d pa[0]=%0h pa[1]=%0h",
                  i, names[i], b_pa[i], b_pa0[i], v_pa0[i], v_pa1[i]);
         chk($sformatf("[%0d] %s $bits(pa)",    i, names[i]), b_pa[i],       8);
         chk($sformatf("[%0d] %s $bits(pa[0])", i, names[i]), b_pa0[i],      4);
         chk($sformatf("[%0d] %s pa[0]",        i, names[i]), int'(v_pa0[i]), 4'h1);
         chk($sformatf("[%0d] %s pa[1]",        i, names[i]), int'(v_pa1[i]), 4'h2);
      end

      $display("TEST packed_port_elem_select: %0d checks, %0d errors -> %s",
               n_checks, n_errors, (n_errors == 0) ? "PASS" : "FAIL");
      $finish;
   end
endmodule
