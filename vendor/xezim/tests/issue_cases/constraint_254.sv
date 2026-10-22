module array_joint_probe ;
   int errors = 0 ;

   task automatic verify(bit ok, string msg);
      if (!ok) begin
         errors++ ;
         $display("FAIL : %s", msg) ;
      end
   endtask

   initial begin
      begin
         int        slot_count ;
         bit [1:0]  parts [] ;
         bit [64:0] words [] ;
         int        total_parts = 12 ;
         if (std::randomize(slot_count, parts, words) with {
            slot_count >= (total_parts >> 1) ;
            slot_count <= (total_parts << 1) ;
            parts.size()   == slot_count ;
            words.size() == slot_count ;
            foreach (parts[i]) { parts[i] inside { [0:2] } ; }
            parts.sum() with (16'(item)) == total_parts ;
         }) begin
            verify(slot_count >= 6 && slot_count <= 24, "joint size outside bounds");
            foreach (parts[i]) verify(parts[i] <= 2, "joint element outside bounds");
            verify(parts.size() == slot_count && words.size() == slot_count, "sum-test: size mismatch") ;
            verify(parts.sum() with (16'(item)) == total_parts, "sum-test: sum mismatch") ;
            $display("sum-test: OK slot_count=%0d parts.size()=%0d words.size()=%0d",
                     slot_count, parts.size(), words.size()) ;
         end else begin
            verify(0, "sum-test: std::randomize returned 0 (spurious)") ;
         end
      end
      begin
         bit [15:0] slot_count ;
         bit [1:0]  parts [] ;
         bit        fixed_mode = 0 ;
         if (std::randomize(slot_count, parts) with {
            if (fixed_mode) {
               slot_count == 5 ;
            } else {
               slot_count >= 6 ;
               slot_count <= 24 ;
            }
            parts.size() == slot_count ;
            foreach (parts[i]) { parts[i] inside { [0:2] } ; }
         }) begin
            verify(slot_count >= 6 && slot_count <= 24, "size-test: slot_count out of range") ;
            foreach (parts[i]) verify(parts[i] <= 2, "sized element outside bounds");
            verify(parts.size() == slot_count, "size-test: parts.size() != slot_count") ;
            $display("size-test: OK slot_count=%0d parts.size()=%0d", slot_count, parts.size()) ;
         end else begin
            verify(0, "size-test: std::randomize returned 0 (spurious)") ;
         end
      end
      if (1) begin
         int       slot_count ;
         bit [1:0] parts [] ;
         bit       fixed_mode = 0 ;
         $display("integer-size case starting") ;
         if (std::randomize(slot_count, parts) with {
            if (fixed_mode) {
               slot_count == 5 ;
            } else {
               slot_count >= 6 ;
               slot_count <= 24 ;
            }
            parts.size() == slot_count ;
            foreach (parts[i]) { parts[i] inside { [0:2] } ; }
         }) begin
            verify(parts.size() == slot_count, "integer size mismatch");
            verify(slot_count >= 6 && slot_count <= 24, "integer size outside bounds");
            foreach (parts[i]) verify(parts[i] <= 2, "integer element outside bounds");
         end
         else verify(0, "hang-test: std::randomize returned 0") ;
      end

      if (errors == 0) $display("TEST_PASS");
      else begin $display("TEST_FAIL count=%0d", errors); $fatal(1); end
      $finish ;
   end
endmodule

