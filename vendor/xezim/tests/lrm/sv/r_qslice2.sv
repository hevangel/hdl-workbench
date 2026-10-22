// top: rqs
module rqs;
  int q[$];
  int n;
  initial begin
    q = {5};
    q = q[1:$];
    $display("T|r1|%p size=%0d", q, q.size());
    q = {1, 2, 3};
    n = 0;
    while (q.size() > 0 && n < 10) begin q = q[1:$]; n++; end
    $display("T|r2|iterations=%0d", n);
  end
endmodule
