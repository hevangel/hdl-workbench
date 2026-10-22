// top: r_progif
interface bif(input logic clk); logic [7:0] d; modport tb(output d, input clk); endinterface
program automatic pg(bif.tb pb);
  initial begin pb.d = 8'h5a; #1 $display("T|24.3if|d=%h", r_progif.b.d); end
endprogram
module r_progif;
  logic clk = 0;
  bif b(clk);
  pg p(b.tb);
  initial #10 $finish;
endmodule
