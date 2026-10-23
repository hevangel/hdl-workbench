
package pkg_one ;
   typedef struct packed {
      bit [1:0] kind ;
      bit [1:0] fetch ;
      bit [1:0] mem_type ;
      bit [1:0] gating_en ;
      bit [3:0] n_units ;
      bit [3:0] first_unit ;
   } mode_t ;

   typedef struct packed {
      bit [31:0] t00 ; bit [31:0] t01 ; bit [31:0] t02 ; bit [31:0] t03 ;
      bit [31:0] t04 ; bit [31:0] t05 ; bit [31:0] t06 ; bit [31:0] t07 ;
      bit [31:0] t08 ; bit [31:0] t09 ; bit [31:0] t10 ; bit [31:0] t11 ;
      bit [31:0] t12 ; bit [31:0] t13 ; bit [31:0] t14 ; bit [31:0] t15 ;
   } thresh_t ;

   // the "small" block: 976 bits, 1-D fields only
   typedef struct packed {
      mode_t   mode ;
      thresh_t thresh ;
      bit [31:0] ctrl_00 ; bit [31:0] ctrl_01 ; bit [31:0] ctrl_02 ;
      bit [31:0] ctrl_03 ; bit [31:0] ctrl_04 ; bit [31:0] ctrl_05 ;
      bit [31:0] ctrl_06 ; bit [31:0] ctrl_07 ; bit [31:0] ctrl_08 ;
      bit [31:0] ctrl_09 ; bit [31:0] ctrl_10 ; bit [31:0] ctrl_11 ;
      bit [31:0] ctrl_12 ; bit [31:0] ctrl_13 ;
   } blk_t ;

   virtual class base_cfg ;
      rand blk_t regs ;
      integer PIN_FIRST_UNIT = -1 ;
      integer PIN_N_UNITS    = -1 ;
      constraint pin_override_c {
         if (PIN_FIRST_UNIT != -1) {
            regs.mode.first_unit == PIN_FIRST_UNIT ;
         }
         if (PIN_N_UNITS != -1) {
            regs.mode.n_units == PIN_N_UNITS ;
         }
      }
   endclass
endpackage

// the "big" block: ANOTHER typedef named blk_t, compiled later, with 2-D
// packed array fields (12288 bits)
package pkg_two ;
   typedef struct packed {
      bit [63:0][23:0]   map_hi ;
      bit [63:0][23:0]   map_lo ;
      bit [31:0][31:0]   cnt_hi ;
      bit [255:0][31:0]  cnt_lo ;
   } blk_t ;
endpackage

module mwe19_typedef_collision_obf ;
   import pkg_one::* ;
   import pkg_two::* ;

   localparam int LIMIT = 2 ;

   class cfg extends base_cfg ;
      rand bit [3:0] sel ;
      constraint n_units_c {
         sel dist { 0 :/ 15, 1 :/ 7, 2 :/ 2, 3 :/ 1 };
         regs.mode.n_units == (4'hf >> sel);
      }
      constraint sum_leq_c {
         ( {1'b0, regs.mode.n_units} +
           {1'b0, regs.mode.first_unit} ) <= (LIMIT - 1) ;
         regs.mode.kind == 0 ;
      }
   endclass

   int fails = 0 ;
   initial begin
      $display("BITS pkg_one::blk_t=%0d  pkg_two::blk_t=%0d",
               $bits(pkg_one::blk_t), $bits(pkg_two::blk_t)) ;
      repeat (3) begin
         automatic cfg c = new() ;
         if (c.randomize())
            $display("randomize OK   : n_units=%0d first_unit=%0d",
                     c.regs.mode.n_units, c.regs.mode.first_unit) ;
         else begin
            $display("randomize FAIL") ;
            fails++ ;
         end
      end
      if (fails == 0) $display("TEST_PASS") ;
      else            $display("TEST_FAIL count=%0d", fails) ;
      $finish ;
   end
endmodule
