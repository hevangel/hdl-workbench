module t;
  logic clk = 0;
  always #5 clk = ~clk;
  int k = 0;
  always_ff @(posedge clk) k <= k + 1;
  logic a, b;
  assign a = (k == 10);
  assign b = (k > 10 && k <= 310);   // 300 ticks of b
  int r299 = 0, r300 = 0, r301 = 0, d300 = 0, d5000 = 0;
  c299: cover property (@(posedge clk) a ##1 b[*299] ##1 b) r299++;
  c300: cover property (@(posedge clk) a ##1 b[*300] ##1 !b) r300++;
  c301: cover property (@(posedge clk) a ##1 b[*301]) r301++;
  cd:   cover property (@(posedge clk) a ##300 (k == 310)) d300++;
  cd2:  cover property (@(posedge clk) a ##5000 (k == 5010)) d5000++;
  initial begin
    #60000;
    $display("RESULT r299=%0d r300=%0d r301=%0d d300=%0d d5000=%0d", r299, r300, r301, d300, d5000);
    $finish;
  end
endmodule
