// top: rq
module rq;
  int q[$];
  int qb[$:3];
  initial begin
    q = {1, 2, 3, 4, 5};
    $display("T|r1|%p", q[3:1]);
    qb = {1, 2, 3, 4, 5, 6};
    $display("T|r2|%p %0d", qb, qb.size());
    qb.push_back(9);
    $display("T|r3|%p", qb);
    qb.push_front(0);
    $display("T|r4|%p", qb);
  end
endmodule
