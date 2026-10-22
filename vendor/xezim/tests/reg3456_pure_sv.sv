// Pure-SystemVerilog regression test for empty-string comparison:
// String comparison with empty strings must return 1, not X.
//
// Root cause: Value::is_equal() returned X when comparing zero-width
// values through the X-handling branch. Fix: early-return 1 for w==0.
//
// Additionally, StringLiteral("") must produce a width-0 value, not
// width-8, to prevent width-mismatch in is_equal from widening the
// zero-width class property to 8 bits where its X bits cause X return.

module test;
   string empty = "";
   string empty2 = "";
   bit eq1 = ("" == "");
   bit eq2 = (empty == "");
   bit eq3 = (empty == empty2);
   bit eq4 = (empty.len() == 0);
   bit eq5 = ("" != "a");

   initial begin
      #1;
      if (eq1 !== 1 || eq2 !== 1 || eq3 !== 1 || eq4 !== 1 || eq5 !== 1) begin
         $write("FAIL: empty string comparison\n");
         $write("  \"\"==\"\": %0d (exp 1)\n", eq1);
         $write("  empty==\"\": %0d (exp 1)\n", eq2);
         $write("  empty==empty2: %0d (exp 1)\n", eq3);
         $write("  empty.len()==0: %0d (exp 1)\n", eq4);
         $write("  \"\"!=\"a\": %0d (exp 1)\n", eq5);
         $fatal(1);
      end
      $write("TAG_PASS\n");
   end

   // Test class property empty string comparison
   // (mirrors uvm_reg_block::m_name pattern)
   class A;
      string m_name = "";
      function string get_name();
         return m_name;
      endfunction
      function bit is_empty();
         return (m_name == "");
      endfunction
   endclass

   A a = new();

   initial begin
      #1;
      if (a.is_empty() !== 1) begin
         $write("FAIL: a.is_empty() = %0d (expected 1)\n", a.is_empty());
         $fatal(1);
      end
      $write("TAG_PASS\n");
   end
endmodule