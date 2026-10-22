// Parent woken by a clock edge at t>0 forks a child, then hops the NBA region.
module top;
  logic clk = 0; int total = 0;
  task wait_nba(); static int nba; static int next_nba; next_nba++; nba <= next_nba; @(nba); endtask
  always #5 clk = ~clk;
  initial begin
    repeat (2) @(posedge clk);
    fork begin total = total + 1; #100; end join_none
    wait_nba();
    $display("T|parent total=%0d t=%0t", total, $time);
    $finish;
  end
endmodule
