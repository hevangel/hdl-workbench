// top: r13a
module r13a;
  function automatic int rsum(int q[$]); if (q.size() == 0) return 0; return q[0] + rsum(q[1:$]); endfunction
  initial $display("T|r1|%0d", rsum('{1, 2, 3, 4}));
endmodule
