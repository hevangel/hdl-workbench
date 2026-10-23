module top;
  int final_ran = 0;
  initial #10 $finish;
  final begin
    final_ran = 1;
    $display("final block");
  end
endmodule
