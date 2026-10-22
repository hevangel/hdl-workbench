module real_window_probe ;
   int errors = 0 ;

   task automatic verify(bit ok, string msg);
      if (!ok) begin
         errors++ ;
         $display("FAIL : %s", msg) ;
      end
   endtask

   initial begin
      bit [15:0] samples [] ;
      int        period = 1000 ;
      int        invalid = 0 ;

      if (std::randomize(samples) with {
         samples.size() == 16 ;
         foreach (samples[i]) {
            samples[i] >= 0.9 * period ;
            samples[i] <= 1.1 * period ;
         }
      }) begin
         verify(samples.size() == 16, "unexpected sample count");
         foreach (samples[i]) begin
            if (samples[i] < 900 || samples[i] > 1100) begin
               invalid++ ;
               $display("out-of-range: samples[%0d]=%0d", i, samples[i]) ;
            end
         end
         verify(invalid == 0, "values outside [900:1100]") ;
         $display("OK size=%0d invalid=%0d first=%0d", samples.size(), invalid, samples[0]) ;
      end else begin
         verify(0, "std::randomize returned 0 (spurious)") ;
      end

      if (errors == 0) $display("TEST_PASS");
      else begin $display("TEST_FAIL count=%0d", errors); $fatal(1); end
      $finish ;
   end
endmodule

