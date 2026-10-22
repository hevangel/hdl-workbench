`timescale 1ns/1ps
`include "includes/top_defs.svh"
module top;
  import marker_pkg::*;
  logic [3:0] count;
  logic [3:0] xz_value;
  logic [7:0] rom [0:1];
  integer fd;
  integer got;
  string line;

  initial begin
    $dumpfile("native-output.vcd");
    $dumpvars(0, top);
    $readmemh("rom.hex", rom);
    count = `INITIAL_COUNT;
    xz_value = 4'b10xz;
    fd = $fopen("captured.txt", "w");
    $fdisplay(fd, "ROM=%0h", rom[1]);
    $fclose(fd);
    fd = $fopen("captured.txt", "r");
    got = $fgets(line, fd);
    $display("FILE=%s", line);
    $fclose(fd);
    #1 count <= count + 1'b1;
    #0 $display("NBA_PRE=%0d", count);
    #1 $display("NBA_POST=%0d", count);
    #1 count <= count + 1'b1;
    #1 $display("ROM=%0h COUNT=%0d", rom[1], count);
    $finish;
  end
endmodule
