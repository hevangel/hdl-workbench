module t;
  initial begin
    // IEEE 1800-2023 5.9 and 11.10: a string literal used as an operand is an
    // unsigned integral value, 8 bits per character, so "z" < "aa" compares
    // 8'h7a with 16'h6161 and is true.
    if ("z" < "aa") $display("PASS");
    else $display("FAIL: (\"z\" < \"aa\") = %b", ("z" < "aa"));
    $finish;
  end
endmodule
