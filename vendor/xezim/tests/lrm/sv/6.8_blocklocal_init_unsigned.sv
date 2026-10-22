// top: run
module run;
  int unsigned m1 = -1;
  int unsigned m2;
  initial begin
    int unsigned l1 = -1;
    int unsigned l2;
    longint unsigned l3 = -1;
    l2 = -1; m2 = -1;
    $display("T|r1|%0d %0d %0d %0d %0d", m1, m2, l1, l2, l3);
    $display("T|r2|%0d", l1 > 0);
    $display("T|r3|%0d %d", m1, m1);
  end
endmodule
