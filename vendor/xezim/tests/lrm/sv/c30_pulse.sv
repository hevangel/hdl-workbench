// top: c30_pulse
`timescale 1ns/1ps
module pd (input a, output y1, y2, y3);
  buf (y1, a); buf (y2, a); buf (y3, a);
  specify
    specparam PATHPULSE$a$y2 = (2, 4);
    (a => y1) = 5;
    (a => y2) = 5;
    showcancelled y3;
    pulsestyle_ondetect y3;
    (a => y3) = (5, 3);
  endspecify
endmodule
module c30_pulse;
  logic a; wire y1, y2, y3;
  pd u (a, y1, y2, y3);
  initial begin
    a = 0; #20 a = 1; #1 a = 0;     // 1ns pulse: rejected
    #20 a = 1; #3 a = 0;           // 3ns pulse: y1 inertial reject; y2 x (between 2 and 4)
    #20 a = 1; #10 a = 0;          // passes
    #20 a = 1; #1 a = 0;           // y3: rise 5 fall 3 -> trailing scheduled before leading -> cancelled
    #30 $finish;
  end
  always @(y1) $display("T|30.7|t=%0t y1=%b", $realtime, y1);
  always @(y2) $display("T|30.7|t=%0t y2=%b", $realtime, y2);
  always @(y3) $display("T|30.7.4|t=%0t y3=%b", $realtime, y3);
endmodule
