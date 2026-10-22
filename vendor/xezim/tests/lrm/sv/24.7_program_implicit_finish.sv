// top: c24x
program automatic p1(input logic clk);
  initial begin
    repeat (2) @(posedge clk);
    $display("T|24.7a|p1 done t=%0t", $time);
  end
endprogram
program automatic p2(input logic clk);
  initial begin
    repeat (4) @(posedge clk);
    $display("T|24.7b|p2 done t=%0t", $time);
  end
  final $display("T|24.7c|p2 final t=%0t", $time);
endprogram
module c24x;
  logic clk = 0;
  always #5 clk = ~clk;
  p1 a(clk);
  p2 b(clk);
  initial begin #1000 $display("T|24.7d|module still running t=%0t (should not print)", $time); end
  final $display("T|24.7e|module final t=%0t", $time);
endmodule
