typedef struct packed { bit [23:0] lo; bit [23:0] hi; } rng_t;
module tb;
  rng_t mfq [3:0] [$];
  rng_t q [$];
  rng_t r [$];
  rng_t t;
  initial begin
    for (int i = 0; i < 4; i++) begin t.lo = i; t.hi = 9; mfq[2].push_back(t); q.push_back(t); end
    r = q.find with (item.lo > 1);
    $display("T|01 plain %0d", r.size());
    r = mfq[2].find with (item.lo > 1);
    $display("T|02 into var %0d", r.size());
    mfq[3] = mfq[2].find with (item.lo > 1);
    $display("T|03 into elem %0d", mfq[3].size());
    mfq[3] = mfq[2].find with (item > 0);
    $display("T|04 into elem nofield %0d", mfq[3].size());
  end
endmodule
