// top: r13a2
module r13a2;
  function automatic int rsum(int q[$]); if (q.size() == 0) return 0; return q[0] + rsum(q[1:$]); endfunction
  function automatic int qsz(int q[$]); return q.size(); endfunction
  int src[$];
  initial begin
    src = {1, 2, 3, 4};
    $display("T|r1|%0d %0d", qsz(src[1:$]), qsz(src[4:$]));
    $display("T|r2|%0d", rsum(src));
  end
endmodule
