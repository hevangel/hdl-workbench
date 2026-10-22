// top: t16_11
module t16_11;
  bit clk, a, b;
  always #5 clk = ~clk;
  initial begin @(negedge clk) a = 1; @(negedge clk) a = 0; b = 1; @(negedge clk) b = 0; repeat (2) @(negedge clk); $finish; end
  function void note(string s); $display("T|m|match %s t=%0t", s, $time); endfunction
  c_sub: cover property (@(posedge clk) (a, note("a")) ##1 (b, note("b")));
endmodule
