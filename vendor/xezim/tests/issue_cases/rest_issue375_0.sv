// Sampled at posedges k = 0..4:  a = 1 1 0 1 1,  b = 1 0 1 1 1
//   a #-# b fails at k = 1, 2 and passes at k = 0, 3, 4
//   a #=# b fails at k = 0, 2 and passes at k = 1, 3 (k = 4 is still pending)
module top;
  logic clk = 0, a, b;
  logic [4:0] av = 5'b11011, bv = 5'b11101;  // bit k is the value at posedge k
  always #5 clk = ~clk;
  int cnt[4];                                   // passes and fails of ov, then of no
  function void count(int k); cnt[k]++; endfunction
  ov: assert property (@(posedge clk) a #-# b) count(0); else count(1);
  no: assert property (@(posedge clk) a #=# b) count(2); else count(3);
  initial begin
    for (int k = 0; k < 5; k++) begin
      a = av[k]; b = bv[k];
      @(posedge clk); @(negedge clk);
    end
  end
  initial begin
    repeat (5) @(posedge clk);
    #1;
    if (cnt[0] == 3 && cnt[1] == 2 && cnt[2] == 2 && cnt[3] == 2) $display("PASS");
    else $display("FAIL #-#: %0d pass %0d fail, #=#: %0d pass %0d fail", cnt[0], cnt[1], cnt[2], cnt[3]);
    $finish;
  end
endmodule
