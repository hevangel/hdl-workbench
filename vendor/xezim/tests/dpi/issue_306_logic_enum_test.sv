typedef enum int { RED = 1, GREEN = 2, BLUE = 3 } color_e ;

module mwe_dpi_combined ;
   int failures = 0 ;

   task automatic chk(bit ok, string msg);
      if (!ok) begin
         failures++ ;
         $display("FAIL : %s", msg) ;
      end
   endtask

   /* (a) C -> SV 4-state marshalling */
   import "DPI-C" function void  dpi_logic_out (output logic b) ;
   import "DPI-C" function void  dpi_logic_zout(output logic b) ;
   import "DPI-C" function logic dpi_logic_ret () ;

   /* (b) enum-typed DPI prototype (LRM 35.5.5) */
   import "DPI-C" function color_e dpi_next_color(input color_e c) ;

   initial begin
      logic lo ;

      // (a) C drives X / Z back into SV
      dpi_logic_out(lo) ;
      $display("C wrote sv_x(3); SV sees %b", lo) ;
      chk(lo === 1'bx, "C->SV: writing sv_x to output logic yields 1 (X lost)") ;

      dpi_logic_zout(lo) ;
      $display("C wrote sv_z(2); SV sees %b", lo) ;
      chk(lo === 1'bz, "C->SV: writing sv_z to output logic yields 0 (Z lost)") ;

      lo = dpi_logic_ret() ;
      $display("C returned sv_x(3); SV sees %b", lo) ;
      chk(lo === 1'bx, "C->SV: returning sv_x yields 1 (X lost)") ;

      // (b) enum round-trip
      chk(dpi_next_color(GREEN) == BLUE, "enum round-trip failed") ;

      if (failures == 0) $display("TEST_PASS");
      else begin $display("TEST_FAIL count=%0d", failures); $fatal(1); end
      $finish ;
   end
endmodule

