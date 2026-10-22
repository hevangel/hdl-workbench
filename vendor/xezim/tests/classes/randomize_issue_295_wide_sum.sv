module mwe11_wide_elem_sum ;
   bit [7:0]  narrow [] ;
   bit [15:0] wide16 [] ;
   int        wide32 [] ;
   int failures = 0 ;
   task automatic chk(bit ok, string msg);
      if (!ok) begin failures++ ; $display("FAIL : %s", msg) ; end
   endtask
   initial begin
      // ---- the failing shape: 16-bit elements, MODULE scope, no cast ----
      if (std::randomize(wide16) with { wide16.size() == 5 ; wide16.sum() == 3 ; })
         chk(wide16.sum() == 3, "wide16: sum mismatch") ;
      else chk(0, "wide16: std::randomize returned 0 (spurious)") ;
      // ---- 32-bit (int) elements, with cast: also fails ----
      if (std::randomize(wide32) with { wide32.size() == 5 ;
                                        wide32.sum() with (32'(item)) == 3 ; })
         chk(wide32.sum() == 3, "wide32: sum mismatch") ;
      else chk(0, "wide32: std::randomize returned 0 (spurious)") ;
      // ---- control: 8-bit elements, same constraints: passes ----
      if (std::randomize(narrow) with { narrow.size() == 5 ; narrow.sum() == 3 ; })
         chk(narrow.sum() == 3, "narrow: sum mismatch") ;
      else chk(0, "narrow: std::randomize returned 0") ;
      if (failures == 0) $display("TEST_PASS");
      else begin $display("TEST_FAIL count=%0d", failures); $fatal(1); end
   end
endmodule
