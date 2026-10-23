// Two hoppers in one slot; the second is queued (not yet started) when the first parks.
module top;
  logic clk = 0; int a = 0, b = 0;
  task wait_nba(); static int nba; static int next_nba; next_nba++; nba <= next_nba; @(nba); endtask
  initial #5 clk = 1;
  initial begin
    @(posedge clk);
    fork
      begin b <= 1; wait_nba(); $display("T|B sees a=%0d b=%0d", a, b); end
    join_none
    a <= 1;
    wait_nba();
    $display("T|A sees a=%0d b=%0d", a, b);
    #10 $finish;
  end
endmodule
