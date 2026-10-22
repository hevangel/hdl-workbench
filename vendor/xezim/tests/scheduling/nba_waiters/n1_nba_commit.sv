// Pre-drained child queues an NBA then hops the NBA region: both must see x=7.
module top;
  logic clk = 0;
  int x = 0;
 
  task wait_nba(); static int nba; static int next_nba; next_nba++; nba <= next_nba; @(nba); endtask
  initial #5 clk = 1;
  initial begin
    @(posedge clk);
    fork
      begin x <= 7; wait_nba(); $display("T|child x=%0d t=%0t", x, $time); end
    join_none
    wait_nba();
    $display("T|parent x=%0d t=%0t", x, $time);
    #10 $finish;
  end
endmodule
