// selfret.sv
typedef byte ba_t[];

module top;
  function automatic ba_t f(int n);
    return f;                 // stack overflow under xezim
  endfunction

  initial begin
    ba_t r;
    r = f(2);
    $display("got %p", r);
  end
endmodule
