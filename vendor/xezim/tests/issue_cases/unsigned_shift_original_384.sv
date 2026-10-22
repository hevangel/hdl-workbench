// acc + (s >>> 1) is unsigned because acc is, so s takes the expression's type
// and width, unsigned 16 bits, and is zero-extended before the shift (IEEE
// 1800-2017 11.8.1, 11.8.2); >>> of an unsigned operand is a logical shift
// (11.4.10). 16'h00FC >>> 1 = 16'h007E, and the same for >>.
module top;
  logic signed [7:0] s = 8'shFC;
  logic [15:0] acc = 16'd0;
  logic [15:0] r1, r2, r3, r4;
  assign r1 = acc + (s >>> 1);
  always_comb r2 = acc + (s >>> 1);
  initial begin
    #1;
    r3 = acc + (s >>> 1);
    r4 = acc + (s >> 1);
    if ({r1, r2, r3, r4} === {4{16'h007E}}) $display("PASS");
    else $display("FAIL assign %h, always_comb %h, initial %h, initial with >> %h (expect 007e)", r1, r2, r3, r4);
  end
endmodule
