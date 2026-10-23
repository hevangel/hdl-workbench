`timescale 1ns/1ps
module dpi_top;
  import "DPI-C" function int add_ints(input int left, input int right);
  import "DPI-C" function int add_ints_cpp(input int left, input int right);

  initial begin
    $dumpfile("dpi-output.vcd");
    $dumpvars(0, dpi_top);
    if (add_ints(10, 20) != 30) $fatal(1, "C DPI result mismatch");
    if (add_ints_cpp(19, 23) != 42) $fatal(1, "C++ DPI result mismatch");
    $display("DPI_C=30");
    $display("DPI_CPP=42");
    $finish;
  end
endmodule
