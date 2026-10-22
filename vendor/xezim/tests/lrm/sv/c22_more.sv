// top: c22m
`timescale 1ns/1ns
`unconnected_drive pull1
module ud(input a, output y); assign y = a; endmodule
`nounconnected_drive
module ud0(input a, output y); assign y = a; endmodule
`pragma my_tool_option
module c22m;
  wire y1, y0;
  ud u1(.a(), .y(y1));
  ud0 u0(.a(), .y(y0));
  `define M1(a) a+1
  `define M2(a) `M1(a)*2
  initial #1 begin
    $display("T|22.9|y1=%b y0=%b", y1, y0);
    $display("T|22.5.1i|%0d", `M2(3));
    $display("T|22.11|pragma ignored");
  end
endmodule
