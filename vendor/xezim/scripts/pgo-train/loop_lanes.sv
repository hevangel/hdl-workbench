
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [15:0][7:0] sbox_o, sbox_i;
  logic [31:0][31:0] mem;
  logic [31:0] db; logic [4:0] ab; int cyc = 0;
  always @(posedge clk) begin
    for (int b = 0; b < 16; b++)
      for (int m = 0; m < 8; m++)
        sbox_o[b][m] = sbox_i[(b + m) & 15][m];
    for (int i = 0; i < 4; i++)
      mem[ab][(i * 8) +: 8] <= db[(i * 8) +: 8];
    cyc <= cyc + 1; ab <= ab + 1; db <= db + 32'h0101_0101;
  end
  initial begin
    sbox_o = '0; mem = '0; db = 32'hdead_beef; ab = 0;
    for (int i = 0; i < 16; i++) sbox_i[i] = 8'(i * 7 + 1);
    repeat (20000) @(posedge clk);
    #1 $display("LANES %h %h %h %0d", sbox_o, mem[7], mem[19], cyc);
    $finish;
  end
endmodule
