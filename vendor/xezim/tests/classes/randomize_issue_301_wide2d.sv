// #301 (reported by Ganesh-TNL): `foreach` over a state vector equating whole
// multi-dimensional packed members wider than 64 bits of two rand
// sub-objects. The reference simulator prints OK on every line.

module mwe20_wide2d_foreach_silent_obf ;
   // 64-bit total: at the threshold -- passes
   class blk_64   ; rand bit [7:0][7:0]   W ; endclass
   // 72-bit total: just past the threshold -- fails
   class blk_72   ; rand bit [8:0][7:0]   W ; endclass
   // 1536-bit total: a wide 2-D packed member -- fails
   class blk_wide ; rand bit [31:0][47:0] W ; endclass

   class t_64 ;   // 2D 8x8  = 64b  -- control, passes
      rand blk_64 a, b ;
      bit [0:0] sel ;
      constraint c { foreach (sel[i]) { b.W == a.W ; } }
      function new() ; a = new() ; b = new() ; endfunction
   endclass
   class t_72 ;   // 2D 9x8  = 72b  -- silent FAIL
      rand blk_72 a, b ;
      bit [0:0] sel ;
      constraint c { foreach (sel[i]) { b.W == a.W ; } }
      function new() ; a = new() ; b = new() ; endfunction
   endclass
   class t_wide ;   // 2D 32x48 = 1536b -- silent FAIL
      rand blk_wide a, b ;
      bit [0:0] sel ;
      constraint c { foreach (sel[i]) { b.W == a.W ; } }
      function new() ; a = new() ; b = new() ; endfunction
   endclass

   int fails = 0 ;
   initial begin
      repeat (2) begin
         begin
            automatic t_64 t = new() ;
            if (t.randomize()) $display("t_64   2D 8x8=64b    OK") ;
            else begin $display("t_64   2D 8x8=64b    FAIL") ; fails++ ; end
         end
         begin
            automatic t_72 t = new() ;
            if (t.randomize()) $display("t_72   2D 9x8=72b    OK") ;
            else begin $display("t_72   2D 9x8=72b    FAIL") ; fails++ ; end
         end
         begin
            automatic t_wide t = new() ;
            if (t.randomize()) $display("t_wide 2D 32x48=1536b OK") ;
            else begin $display("t_wide 2D 32x48=1536b FAIL") ; fails++ ; end
         end
      end
      if (fails == 0) $display("TEST_PASS") ;
      else            $display("TEST_FAIL count=%0d", fails) ;
      $finish ;
   end
endmodule
