// top: c28_delay
`timescale 1ns/1ps
module c28_delay;
  logic a, en; wire o1, o2, o3, o4, o5;
  buf #(3) g1 (o1, a);
  buf #(2, 5) g2 (o2, a);
  bufif1 #(1, 2, 4) g3 (o3, a, en);
  buf #(1:2:3) g4 (o4, a);
  not #(2) g5 (o5, a);
  initial begin
    a = 0; en = 1;
    #10 a = 1; #10 a = 0; #10 a = 1; #1 a = 0;   // 1-tick pulse shorter than delay: inertial
    #10 en = 0; #10 en = 1; #10 a = 1'bx; #10 a = 1;
    #20 $finish;
  end
  always @(o1) $display("T|28.16|t=%0t o1=%b", $realtime, o1);
  always @(o2) $display("T|28.16|t=%0t o2=%b", $realtime, o2);
  always @(o3) $display("T|28.16|t=%0t o3=%b", $realtime, o3);
  always @(o4) $display("T|28.16|t=%0t o4(typ)=%b", $realtime, o4);
  always @(o5) $display("T|28.16|t=%0t o5=%b", $realtime, o5);
endmodule
