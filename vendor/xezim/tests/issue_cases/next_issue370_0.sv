// A packed struct member read by a concurrent assertion is sampled in the
// Preponed region (IEEE 1800-2017 16.5.1), like a whole variable.
// Expected output: DONE, with no "failed" line before it.
module t;
  typedef struct packed { logic [1:0] a; logic b; } s_t;
  logic clk = 0, go = 0;
  s_t s = '0;
  logic [1:0] v = 0;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    s.a <= (s.a == 0 && go) ? 2'd1 : 2'd0;   // 0 -> 1 at 15 ns, back to 0 at 25 ns
    v   <= (v == 0 && go) ? 2'd1 : 2'd0;     // the same, in a plain variable
  end
  // go is sampled 1 at 15 ns, so at 25 ns s.a and v are sampled 1.
  a_s: assert property (@(posedge clk) go |=> s.a == 1) else $display("FAIL: a_s failed at %0t", $time);
  a_v: assert property (@(posedge clk) go |=> v == 1) else $display("FAIL: a_v failed at %0t", $time);
  initial begin
    @(negedge clk) go = 1;
    @(negedge clk) go = 0;
    repeat (3) @(negedge clk);
    $display("DONE");
    $finish;
  end
endmodule
