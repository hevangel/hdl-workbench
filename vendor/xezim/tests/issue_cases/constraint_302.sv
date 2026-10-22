// Regression for issue #302: a joint solve whose constraint ties an int to an
// expression forced unsigned by a 32'(...) cast, and pins an array sum inside
// a window whose center reads a rand variable, used to exhaust the solver's
// work budget and fail std::randomize even though the problem is feasible.
//
//   * RC1: `v0 == (v2 * 32'(p0) * (p1 + 1))` is an unsigned comparison; the
//     signed operand's zero-extension "v0 >= 0" fit kept the modular-equality
//     fallback from ever running, so v0 stayed 2^32 wide.
//   * RC2: `v1.sum() inside {[v0-10 : v0+10]}` with a rand `v0` was lowered to
//     two relations that each re-translated the sum into its own carry pair;
//     the two independent wrap counts refuted elements one at a time.
module constraint_302_probe ;
   int errors = 0 ;

   task automatic verify(bit ok, string msg);
      if (!ok) begin
         $display("FAIL : %s", msg) ;
         errors++ ;
      end
   endtask

   function automatic bit in_window(int sum, int centre, int w);
      in_window = (sum >= centre - w) && (sum <= centre + w) ;
   endfunction

   initial begin
      begin
         bit [15:0] p0 = 16'd256 ;
         bit [1:0]  p1 = 3 ;
         int        v2 = 17 ;
         int        v1 [(16 + 1)] ;
         int        v0 ;
         foreach (v1[i]) v1[i] = 1000 + (i % 37) ;
         if (std::randomize(v0, v1) with {
            v0 == (v2 * 32'(p0) * (p1 + 1)) ;
            v1.sum() with (32'(item)) inside { [v0 - 10:v0 + 10] } ;
         }) begin
            int sum = 0 ;
            foreach (v1[i]) sum += 32'(v1[i]) ;
            verify(v0 == (v2 * 32'(p0) * (p1 + 1)), "issue#302: v0 equality not met") ;
            verify(in_window(sum, v0, 10), "issue#302: sum outside window") ;
         end else begin
            verify(0, "issue#302: std::randomize returned 0 (spurious)") ;
         end
      end
      // Same sum-window with the equality replaced by a literal: the rand
      // bound (RC2) alone must still solve.
      begin
         bit [15:0] p0 = 16'd256 ;
         bit [1:0]  p1 = 3 ;
         int        v1 [(16 + 1)] ;
         int        v0 ;
         foreach (v1[i]) v1[i] = 1000 + (i % 37) ;
         if (std::randomize(v0, v1) with {
            v0 == 17408 ;
            v1.sum() with (32'(item)) inside { [v0 - 10:v0 + 10] } ;
         }) begin
            int sum = 0 ;
            foreach (v1[i]) sum += 32'(v1[i]) ;
            verify(v0 == 17408, "issue#302: literal v0 not met") ;
            verify(in_window(sum, v0, 10), "issue#302: literal sum outside window") ;
         end else begin
            verify(0, "issue#302: literal std::randomize returned 0 (spurious)") ;
         end
      end
      // The same window written as two separate relations on the sum (issue
      // #302, C2). Each item re-translates `v1.sum()`; they must share the one
      // modular identity (`sum >= lo`, `sum <= hi`), or the two independent
      // carry pairs refute elements one at a time.
      begin
         bit [15:0] p0 = 16'd256 ;
         bit [1:0]  p1 = 3 ;
         int        v2 = 17 ;
         int        v1 [(16 + 1)] ;
         int        v0 ;
         foreach (v1[i]) v1[i] = 1000 + (i % 37) ;
         if (std::randomize(v0, v1) with {
            v0 == (v2 * 32'(p0) * (p1 + 1)) ;
            v1.sum() with (32'(item)) >= v0 - 10 ;
            v1.sum() with (32'(item)) <= v0 + 10 ;
         }) begin
            int sum = 0 ;
            foreach (v1[i]) sum += 32'(v1[i]) ;
            verify(v0 == (v2 * 32'(p0) * (p1 + 1)), "issue#302: C2 v0 equality not met") ;
            verify(in_window(sum, v0, 10), "issue#302: C2 sum outside window") ;
         end else begin
            verify(0, "issue#302: C2 std::randomize returned 0 (spurious)") ;
         end
      end
      if (errors == 0) $display("TEST_PASS");
      else begin $display("TEST_FAIL count=%0d", errors); $fatal(1); end
      $finish ;
   end
endmodule