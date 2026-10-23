// top: c04r
program automatic tbp(input logic clk, input logic [3:0] q, output logic [3:0] d);
  initial begin
    d = 0;
    repeat (3) begin
      @(posedge clk);
      // reactive region: NBA updates from the design are visible
      $display("T|4.4.2.6a|prog t=%0t q=%0d", $time, q);
      d = d + 1;
    end
    #100;
  end
endprogram
module c04r;
  logic clk = 0;
  logic [3:0] q = 0, d;
  logic [3:0] q2;
  always #5 clk = ~clk;
  always_ff @(posedge clk) q <= d;
  tbp p(clk, q, d);
  // module initial sees the pre-NBA value (active region)
  initial begin
    repeat (3) begin
      @(posedge clk);
      $display("T|4.4.2.2a|mod t=%0t q=%0d", $time, q);
    end
  end
  // preponed sampling via $sampled
  always @(posedge clk) if ($time < 30) $display("T|4.4.2.1|t=%0t sampled=%0d now=%0d", $time, $sampled(q), q);
  // #0 vs NBA from another process
  logic r = 0;
  initial begin
    #40;
    r <= 1;
    #0;
    $display("T|4.4.2.3c|r after #0 = %b", r);
  end
  // $strobe vs $display in two processes at same time
  logic s = 0;
  initial begin #50 s = 1; $strobe("T|4.4.2.9c|strobe s=%b", s); end
  initial begin #50 #0 s = 0; end
  // event triggered in NBA region by NBA-updated variable wakes waiters
  logic n = 0; int woke = 0;
  initial begin #60 n <= 1; end
  initial begin @(posedge n) woke = $time; #1 $display("T|4.4.2.4|woke=%0d", woke); end
  // continuous assign and its reader in same time step see consistent value after #0? (nondeterministic: compare at #1)
  initial #80 $finish;
endmodule
