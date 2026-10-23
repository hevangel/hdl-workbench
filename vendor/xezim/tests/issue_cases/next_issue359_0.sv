module t;
  parameter [1:0] P = 2'b11;
  logic [1:0] p = 2'b11;
  initial begin
    // IEEE 1800-2023 11.8.1: unary minus keeps its operand's type, so -p is
    // unsigned. 11.8.2: a relational operator with an unsigned operand
    // compares unsigned, both sides extended to 32 bits: -p is 32'hFFFF_FFFD,
    // which is not less than 0.
    if ((-p < 0) == 1'b0) $display("PASS variable");
    else $display("FAIL variable: (-p < 0) = %b", (-p < 0));
    if ((-P < 0) == 1'b0) $display("PASS parameter");
    else $display("FAIL parameter: (-P < 0) = %b", (-P < 0));
    $finish;
  end
endmodule
