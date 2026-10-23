// top: t16_3b
module t16_3b;
  int a, b, c, d;
  always_comb begin
    ai: assert (a >= b) $display("T|i|pass a=%0d b=%0d t=%0t", a, b, $time); else $display("T|i|FAIL a=%0d b=%0d t=%0t", a, b, $time);
  end
  always_comb begin
    c = a;
    af: assert final (a >= b) $display("T|f|pass a=%0d b=%0d t=%0t", a, b, $time); else $display("T|f|FAIL a=%0d b=%0d t=%0t", a, b, $time);
  end
  always @* begin
    as: assert (a + 1 > b) else $display("T|s|FAIL a=%0d b=%0d t=%0t", a, b, $time);
  end
  initial begin
    #1 a = 2; b = 1;
    #1 a = 3; b = 3;
    #1 a = 1; #0 b = 1;
    #1 a = 0; b = 5;
    #1 $finish;
  end
endmodule
