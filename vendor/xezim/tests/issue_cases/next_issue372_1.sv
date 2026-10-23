module top;
  int n = 0;
  initial begin
    #5;
    $finish;
    n++;
  end
  final $display("initial: n=%0d (expect 0)", n);
endmodule
