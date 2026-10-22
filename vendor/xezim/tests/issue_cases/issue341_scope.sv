// An `int` declared in a for-loop header is compared as UNSIGNED once
// the process has resumed from a delay inside the loop body. A down-counting
// `i >= 0` loop never ends (i = 4294967289 = -7 read as unsigned), and
// `i > -1` stops after one iteration. Declaring `int i;` before the loop, or
// removing the delay, gives the right 4 iterations.
module top;
  initial begin
    int n;
    int j;

    n = 0;
    for (int i = 3; i >= 0; i--) begin  // expect 4 iterations
      n++;
      if (n > 10) begin
        $display("for (int i...; i >= 0; ...) with #1: RUNAWAY, i=%0d", i);
        break;
      end
      #1;
    end
    $display("for (int i = 3; i >= 0; i--) { #1 }: %0d iterations (expect 4)", n);

    begin : gt
      int m = 0;
      for (int i = 3; i > -1; i--) begin
        m++;
        #1;
      end
      $display("for (int i = 3; i > -1; i--) { #1 }: %0d iterations (expect 4)", m);
      if (n == 4 && m == 4) $display("PASS");
      else $display("FAIL");
    end

    n = 0;
    for (j = 3; j >= 0; j--) begin  // reference: same loop, j declared outside
      n++;
      #1;
    end
    $display("reference, int j declared outside: %0d iterations", n);
    $finish;
  end
endmodule
