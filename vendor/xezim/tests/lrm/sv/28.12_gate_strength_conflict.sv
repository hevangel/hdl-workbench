// top: t28_12
module t28_12;
  logic a, b; wire w5, w8; wire pu;
  buf (weak0, weak1) bw (w5, a); buf (strong0, strong1) bs (w5, b);
  pullup (pu); assign (pull0, pull1) pu = a;
  assign (weak0, weak1) w8 = a; bufif1 (strong0, strong1) bi (w8, b, 1'b1);
  initial begin
    a = 0; b = 1; #1 $display("T|a|a=0 b=1 w5=%b/%v w8=%b/%v (strong 1 should win)", w5, w5, w8, w8);
    a = 1; b = 0; #1 $display("T|b|a=1 b=0 w5=%b/%v w8=%b/%v (strong 0 should win)", w5, w5, w8, w8);
    a = 0; #1 $display("T|c|pullup vs pull0 pu=%b/%v (equal strength -> x)", pu, pu);
  end
endmodule
