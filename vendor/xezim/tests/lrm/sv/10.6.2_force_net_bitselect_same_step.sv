// top: ffb2
module ffb2;
  logic [3:0] a = 0, b = 0;
  wire [7:0] w;
  assign w = {a, b};
  initial begin
    #1 force w[0] = 1'b1;
    a = 4'hf;
    #1 $display("T|10.6.2|w=%b", w);
  end
endmodule
