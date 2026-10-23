// top: t24b
// region ordering: module vs program processes at the same edge
program pg(input logic clk);
  initial forever begin
    @(posedge clk);
    $display("T|24.ord|t=%0t program", $time);
    if ($time > 20) break;
  end
  final $display("T|24.7f|program final");
endprogram
module t24b;
  logic clk = 0;
  always #5 clk = ~clk;
  always @(posedge clk) $display("T|24.ord|t=%0t module active", $time);
  always @(posedge clk) #0 $display("T|24.ord|t=%0t module inactive", $time);
  logic q = 0;
  always @(posedge clk) q <= ~q;
  always @(q) $display("T|24.ord|t=%0t module after nba q=%b", $time, q);
  pg p(clk);
  final $display("T|24.7f|module final t=%0t", $time);
  initial #30 $finish;
endmodule
