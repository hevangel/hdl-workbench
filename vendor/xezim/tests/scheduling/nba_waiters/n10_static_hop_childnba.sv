// Child writes with an NBA - commits in the same NBA region as the hop.
module top;
  int total = 0;
  task wait_nba(); static int nba; static int next_nba; next_nba++; nba <= next_nba; @(nba); endtask
  initial begin
    fork begin total <= total + 1; #10; end join_none
    wait_nba();
    $display("T|parent total=%0d", total);
  end
endmodule
