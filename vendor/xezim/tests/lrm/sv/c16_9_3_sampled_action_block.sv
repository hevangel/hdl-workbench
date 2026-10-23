// §16.9.3 / §16.5.1: `$sampled(e)` in a concurrent assertion's action block
// returns the sampled (Preponed, slot-entry) value of `e` even when the
// property itself does not read `e`. `q` is set by a nonblocking update in
// the same slot the action runs in; the sampled value must come from before
// that update, so q samples as 0, not the live 1 (issue #468).
module t;
  logic clk = 0, q = 0;
  always #5 clk = ~clk;
  always_ff @(posedge clk) q <= 1;
  a: assert property (@(posedge clk) 1) begin
       if ($sampled(q) === 1'b0)
         $display("T|16.9.3|sampled-action t=%0t q=%0d", $time, $sampled(q));
       else
         $display("T|16.9.3|sampled-action FAIL q=%0d", $sampled(q));
       $finish;
     end
endmodule