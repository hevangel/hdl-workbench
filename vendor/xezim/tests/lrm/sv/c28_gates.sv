// top: c28_gates
module c28_gates;
  logic a, b, c;
  wire w_and, w_nand, w_or, w_nor, w_xor, w_xnor, w_and3, w_buf1, w_buf2, w_not;
  wire w_bif1, w_bif0, w_nif1, w_nif0;
  and  g1 (w_and, a, b);  nand g2 (w_nand, a, b); or g3 (w_or, a, b); nor g4 (w_nor, a, b);
  xor  g5 (w_xor, a, b);  xnor g6 (w_xnor, a, b); and g7 (w_and3, a, b, c);
  buf  g8 (w_buf1, w_buf2, a); not g9 (w_not, a);
  bufif1 g10 (w_bif1, a, c); bufif0 g11 (w_bif0, a, c); notif1 g12 (w_nif1, a, c); notif0 g13 (w_nif0, a, c);
  logic vals[4] = '{1'b0, 1'b1, 1'bx, 1'bz};
  initial begin
    c = 1;
    foreach (vals[i]) foreach (vals[j]) begin
      a = vals[i]; b = vals[j]; #1;
      $display("T|28.4|a=%b b=%b and=%b nand=%b or=%b nor=%b xor=%b xnor=%b and3=%b buf=%b%b not=%b", a, b, w_and, w_nand, w_or, w_nor, w_xor, w_xnor, w_and3, w_buf1, w_buf2, w_not);
    end
    foreach (vals[i]) foreach (vals[j]) begin
      a = vals[i]; c = vals[j]; #1;
      $display("T|28.5|d=%b ctl=%b bif1=%b/%v bif0=%b/%v nif1=%b/%v nif0=%b/%v", a, c, w_bif1, w_bif1, w_bif0, w_bif0, w_nif1, w_nif1, w_nif0, w_nif0);
    end
  end
endmodule
