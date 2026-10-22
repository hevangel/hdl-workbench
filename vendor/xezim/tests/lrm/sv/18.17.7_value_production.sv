// top: t18_17
module t18_17;
  int n;
  initial begin
    $display("T|a|before");
    randsequence (vr)
      void vr : r1 r2 { n = r1 + r2; };
      int r1 : { return 5; };
      int r2 : { return 6; };
    endsequence
    $display("T|b|after n=%0d", n);
  end
endmodule
