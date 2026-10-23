// Edge-woken parent; join_none child hits #0 before writing.
module top;
  logic clk = 0; int total = 0;
  task wait_nba(); static int nba; static int next_nba; next_nba++; nba <= next_nba; @(nba); endtask
  initial #5 clk = 1;
  initial begin
    @(posedge clk);
    fork begin #0; total = total + 1; #10; end join_none
    wait_nba();
    $display("T|parent total=%0d t=%0t", total, $time);
  end
endmodule
