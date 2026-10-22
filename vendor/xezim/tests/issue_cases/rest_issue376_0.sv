module top;
  function automatic logic neg(logic signed [7:0] s);
    return s < 0;
  endfunction
  logic [7:0] u = 8'hFE;
  logic r1, r2, r3;
  always_comb r1 = neg(u);
  always_comb r2 = neg(8'hFE);
  initial begin
    #1;
    r3 = neg(u);
    if (r1 === 1'b1 && r2 === 1'b1 && r3 === 1'b1) $display("PASS");
    else $display("FAIL always_comb neg(u)=%b, always_comb neg(8'hFE)=%b, initial neg(u)=%b", r1, r2, r3);
  end
endmodule
