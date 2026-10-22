// top: rcb
`timescale 1ns/1ps
module rcb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] d = 0, dout;
  always @(posedge clk) d <= d + 1;
  clocking cb @(posedge clk);
    default input #1step output #2;
    input d;
    input #3 dl = d;
    output dout;
  endclocking
  initial begin
    repeat (4) begin
      @(cb);
      $display("T|r1|t=%0t cb.d=%0d cb.dl=%0d d=%0d", $time, cb.d, cb.dl, d);
    end
    repeat (2) begin
      @(posedge clk);
      $display("T|r2|t=%0t cb.d=%0d d=%0d", $time, cb.d, d);
    end
    cb.dout <= 8'hab;
    #1 $display("T|r3|t=%0t dout=%h", $time, dout);
    #2 $display("T|r4|t=%0t dout=%h", $time, dout);
    $finish;
  end
endmodule
