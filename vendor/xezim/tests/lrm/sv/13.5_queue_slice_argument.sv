// top: r13a3
module r13a3;
  function automatic int qsz(int q[$]); return q.size(); endfunction
  int src[$];
  initial begin
    src = {1, 2, 3, 4};
    $display("T|r1|%0d", qsz(src[1:$]));
    $display("T|r2|%0d", qsz(src[3:$]));
    $display("T|r3|%0d", qsz(src[4:$]));
  end
endmodule
