// a fixed-size unpacked dimension sized by a class-scoped localparam
// (cls::N) elaborates to ZERO elements. Reads return a 32-bit 0, foreach
// over it is a no-op, and randomize() "succeeds" leaving every element 0.
// A literal size works.
class model_c;
  localparam int ROWS = 145;
  localparam int BEATS = (ROWS + 1) / 2;  // 73
endclass

class seq_c;
  rand logic [7:0] by_class_param[2 * model_c::BEATS];  // expect 146 elements
  rand logic [7:0] by_literal[146];                     // reference
endclass

module top;
  initial begin
    seq_c s = new();
    $display("$size(by_class_param) = %0d  (expect 146)", $size(s.by_class_param));
    $display("$size(by_literal)     = %0d  (expect 146)", $size(s.by_literal));
    $display("$bits(by_class_param[0]) = %0d, value printed: %h  (expect 8, 2 hex digits)",
             $bits(s.by_class_param[0]), s.by_class_param[0]);
    
    if (!s.randomize() with {
      foreach (by_class_param[i]) by_class_param[i] == 8'(i + 1);
    })

    $display("randomize() returned 0");
    $display("by_class_param[0..2] = %h %h %h  (expect 01 02 03)",
             s.by_class_param[0], s.by_class_param[1], s.by_class_param[2]);

    if ($size(s.by_class_param) == 146 && s.by_class_param[2] == 8'h03) $display("PASS");
    else $display("FAIL");
    $finish;
  end
endmodule
