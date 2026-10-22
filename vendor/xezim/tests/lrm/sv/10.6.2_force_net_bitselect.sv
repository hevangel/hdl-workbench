// top: rfb
module rfb;
  logic [3:0] a = 4'hf, b = 4'h0;
  wire [7:0] w;
  assign w = {a, b};
  initial begin
    #1 force w[0] = 1'b1;
    force w[7:6] = 2'b00;
    #1 $display("T|r1|w=%b", w);
    release w[0];
    #1 $display("T|r2|w=%b", w);
    release w[7:6];
    #1 $display("T|r3|w=%b", w);
  end
endmodule
