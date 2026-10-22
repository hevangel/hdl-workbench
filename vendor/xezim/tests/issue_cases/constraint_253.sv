module wide_guard_probe ;
   int errors = 0 ;

   task automatic verify(bit ok, string msg);
      if (!ok) begin
         errors++ ;
         $display("FAIL : %s", msg) ;
      end
   endtask

   initial begin
      logic [255:0] storage_word ;
      bit           clear_enable = 1 ;

      if (std::randomize(storage_word) with {
         clear_enable -> (storage_word == 0) ;
      }) begin
         verify(storage_word == 0, "line not zero although predicate is 1") ;
         $display("OK line_zero=%0d", (storage_word == 0)) ;
      end else begin
         verify(0, "std::randomize returned 0 (spurious)") ;
      end
      begin
         logic [255:0] spare_word ;
         bit           spare_enable = 0 ;
         if (std::randomize(spare_word) with {
            spare_enable -> (spare_word == 0) ;
         }) begin
            verify(!$isunknown(spare_word), "randomized word contains unknown bits");
            $display("OK control (pred=0) spare_word[0]=%b", spare_word[0]);
         end
         else verify(0, "control (pred=0) returned 0") ;
      end

      if (errors == 0) $display("TEST_PASS");
      else begin $display("TEST_FAIL count=%0d", errors); $fatal(1); end
      $finish ;
   end
endmodule

