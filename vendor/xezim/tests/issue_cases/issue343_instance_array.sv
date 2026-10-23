module leaf(input logic [1:0] d, output logic [1:0] q);
  assign q = d;
  initial $display("instance %m");
endmodule
module top;
  logic [7:0] d, q;
  leaf um[1:0][1:0] (.d(d), .q(q));
  initial begin
    d = 8'b11_10_01_00;
    #1;
    // The instance with all right-hand indices takes the least significant
    // slice, iterating the rightmost dimension first (§23.3.3.5).
    if (um[0][0].d === 2'd0 && um[0][1].d === 2'd1 && um[1][0].d === 2'd2 && um[1][1].d === 2'd3)
      $display("PASS 2d elements");
    else
      $display("FAIL 2d elements %0d %0d %0d %0d", um[0][0].d, um[0][1].d, um[1][0].d, um[1][1].d);
    if (q === d) $display("PASS 2d q"); else $display("FAIL 2d q=%b", q);
  end
endmodule
