// IEEE 1800-2017 27.6: an unnamed generate block is named genblk<n>, with n
// the number of its generate construct in the scope (1 here).
module top;
  for (genvar k = 0; k < 2; k++) begin
    logic unl = k;
  end
  initial begin
    $dumpfile("unnamed_genblk.vcd");
    $dumpvars(0, top);
    #1;
    if (genblk1[0].unl === 1'b0 && genblk1[1].unl === 1'b1 && $bits(genblk1[1].unl) == 1) $display("PASS");
    else $display("FAIL genblk1[0].unl=%b genblk1[1].unl=%b", genblk1[0].unl, genblk1[1].unl);
  end
endmodule
