`timescale 1ns/1ns
module top;
  bit clk = 0;
  int n = 2;
  int acc = 0;
  int visits = 0;
  int edges = 0;
  function automatic int f(input int k); return k + 1; endfunction
  task automatic t(input int k); acc = acc + k; endtask
  always #1 clk = ~clk;
  always @(posedge clk) begin
    int i;
    i = 0;
    while (i < 4) begin
      if (i == 3) break;
      i = i + 1;
      if (i == 1) continue;
      begin int k; k = 0; while (1) begin if (k == 2) break; k = k + 1; for (int j = 0; j < n; j++) t(j); end end
      visits = visits + 1;
    end
    edges = edges + 1;
  end
  initial begin
    #4;
    $display("T|visits=%0d edges=%0d acc=%0d", visits, edges, acc);
    if (visits !== 4 || edges !== 2) $fatal(1, "T|FAIL");
    $display("T|LEAK_PASS");
    $finish;
  end
endmodule
