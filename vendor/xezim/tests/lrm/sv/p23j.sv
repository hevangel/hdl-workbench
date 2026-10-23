// top: t23j
module leaf #(parameter P = 0) (output int o); assign o = P; endmodule
module gm #(parameter SEL = 0);
  if (SEL == 1) begin : g leaf #(11) l(.o()); end
  else begin : g leaf #(22) l(.o()); end
endmodule
module outer #(parameter OP = 3);
  int ov = 4;
  module inner_m;
    int nv;
    initial begin #1 nv = OP * 10 + ov; $display("T|23.4|nested sees outer: %0d", nv); end
  endmodule
  inner_m im1(), im2();
endmodule
module t23j;
  int o1, o2, o3;
  leaf #() a(o1), b(.o(o2));
  leaf #(.P(5)) c(o3);
  outer #(7) ou();
  gm g1();
  defparam g1.SEL = 1;          // defparam that changes a generate condition
  initial #2 $display("T|23.10.4|o=%0d %0d %0d gen=%0d", o1, o2, o3, g1.g.l.o);
endmodule
