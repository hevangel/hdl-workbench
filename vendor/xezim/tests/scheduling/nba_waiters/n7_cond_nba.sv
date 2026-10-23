// wait(cond) satisfied by an edge-woken blocking write, with a peer hopping the NBA region.
module top;
  logic clk = 0; int flag = 0, got = 0;
  task wait_nba(); static int nba; static int next_nba; next_nba++; nba <= next_nba; @(nba); endtask
  always #5 clk = ~clk;
  initial begin @(posedge clk); flag = 1; wait_nba(); $display("T|hopper got=%0d t=%0t", got, $time); end
  initial begin wait (flag != 0); got = 1; $display("T|waiter t=%0t", $time); end
  initial #40 $finish;
endmodule
