// Child reaches its write only after #0 (Inactive) - must still precede the NBA hop.
module top;
  int total = 0;
  task wait_nba(); static int nba; static int next_nba; next_nba++; nba <= next_nba; @(nba); endtask
  initial begin
    fork begin #0; total = total + 1; #10; end join_none
    wait_nba();
    $display("T|parent total=%0d", total);
  end
endmodule
