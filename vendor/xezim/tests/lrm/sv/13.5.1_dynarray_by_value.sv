// top: fdv
module fdv;
  function automatic void modv(int d[]); d[0] = 99; endfunction
  function automatic void modq(int q[$]); q[0] = 98; q.push_back(5); endfunction
  int d[]; int q[$];
  initial begin
    d = '{1, 2}; q = {1, 2};
    modv(d); modq(q);
    $display("T|13.5.1d|d=%p q=%p", d, q);
  end
endmodule
