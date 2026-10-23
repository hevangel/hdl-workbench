// top: c20s
module c20s;
  initial begin
    $info("T|20.10a|info %0d", 1);
    $warning("T|20.10b|warn %s", "w");
    $error("T|20.10c|err");
    $error;
    $display("T|20.10d|after error");
    $system("echo T\\|20.17\\|system_ran");
    #5 $fatal(1, "T|20.10e|fatal msg %0d", 5);
    $display("T|20.10f|after fatal (must not print)");
  end
  final $display("T|20.2d|final after fatal t=%0t", $time);
endmodule
