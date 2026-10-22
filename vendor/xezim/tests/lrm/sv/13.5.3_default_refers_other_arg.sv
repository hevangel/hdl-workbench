// top: rad
module rad;
  function automatic int defs(int a = 1, int b = a + 1, int c = 10); return a * 100 + b * 10 + c; endfunction
  function automatic int d2(int a = 1, int b = 2, int c = 3); return a * 100 + b * 10 + c; endfunction
  initial begin
    $display("T|r1|%0d %0d %0d", defs(), defs(, , 7), defs(.c(1), .a(4)));
    $display("T|r2|%0d %0d %0d", d2(, , 7), d2(.c(1), .a(4)), d2(5, , 6));
  end
endmodule
