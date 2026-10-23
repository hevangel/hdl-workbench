// top: rng
module rng;
  logic [3:0] u4 = 4'd3;
  logic [7:0] r8;
  initial begin
    $display("T|r1|%0d %0d", -4'd3, -u4);
    $display("T|r2|%b", -u4 > 4'd5);
    r8 = -u4; $display("T|r3|%0d", r8);
    $display("T|r4|%0d", -8'd1);
    $display("T|r5|%0d", $bits(-u4));
  end
endmodule
