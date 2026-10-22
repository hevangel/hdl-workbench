// IEEE 1800-2023 §23.3.3 -- a port's size is fixed by its own declaration.
// When the actual is narrower or wider than the formal, the CONNECTION
// zero-extends or truncates; the formal itself keeps its declared width.
// `$bits(pa)` inside the child is therefore always 8 here, and the element
// stride stays 4.

`timescale 1ns/1ps

module width_probe (
   input  logic [1:0][3:0] pa,
   output int              o_bits_pa,
   output int              o_bits_pa0,
   output logic [7:0]      o_pa
);
   assign o_bits_pa  = $bits(pa);
   assign o_bits_pa0 = $bits(pa[0]);
   assign o_pa       = pa;
endmodule

module packed_port_formal_width;

   logic [3:0]  d_narrow;   // 4 bits  -> zero-extended to 8
   logic [15:0] d_wider;    // 16 bits -> truncated to the low 8

   int         b_pa  [0:2];
   int         b_pa0 [0:2];
   logic [7:0] v_pa  [0:2];

   width_probe u_narrow (.pa(d_narrow), .o_bits_pa(b_pa[0]), .o_bits_pa0(b_pa0[0]), .o_pa(v_pa[0]));
   width_probe u_wider  (.pa(d_wider),  .o_bits_pa(b_pa[1]), .o_bits_pa0(b_pa0[1]), .o_pa(v_pa[1]));
   width_probe u_unconn (              .o_bits_pa(b_pa[2]), .o_bits_pa0(b_pa0[2]), .o_pa(v_pa[2]));

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
      d_narrow = 4'h5;
      d_wider  = 16'hBA21;
      #1;

      $display("TEST packed_port_formal_width");
      $display("  narrow driver logic[3:0]=5     : $bits(pa)=%0d $bits(pa[0])=%0d pa=%0h", b_pa[0], b_pa0[0], v_pa[0]);
      $display("  wider  driver logic[15:0]=ba21 : $bits(pa)=%0d $bits(pa[0])=%0d pa=%0h", b_pa[1], b_pa0[1], v_pa[1]);
      $display("  unconnected                    : $bits(pa)=%0d $bits(pa[0])=%0d", b_pa[2], b_pa0[2]);

      // The formal is 8 bits wide with a 4-bit element stride in all cases.
      chk("narrow: $bits(pa)",    b_pa[0],  8);
      chk("narrow: $bits(pa[0])", b_pa0[0], 4);
      chk("narrow: pa",           int'(v_pa[0]), 8'h05);   // zero-extended

      chk("wider: $bits(pa)",     b_pa[1],  8);
      chk("wider: $bits(pa[0])",  b_pa0[1], 4);
      chk("wider: pa",            int'(v_pa[1]), 8'h21);   // truncated to low 8

      chk("unconn: $bits(pa)",    b_pa[2],  8);
      chk("unconn: $bits(pa[0])", b_pa0[2], 4);

      $display("TEST packed_port_formal_width: %0d checks, %0d errors -> %s",
               n_checks, n_errors, (n_errors == 0) ? "PASS" : "FAIL");
      $finish;
   end
endmodule
