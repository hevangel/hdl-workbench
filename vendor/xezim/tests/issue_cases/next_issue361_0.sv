module tb;
  logic [3:0] r;  // never assigned: all x
  logic eq0, eq7;
  // IEEE 1800-2023 6.24.1: a cast to a 2-state type maps x and z to 0,
  // so int'(r) is 0 and the comparisons are known.
  assign eq0 = (int'(r) == 0);
  assign eq7 = (int'(r) == 7);
  initial begin
    #1;
    if (eq0 === 1'b1) $display("PASS assign eq0"); else $display("FAIL assign eq0=%b", eq0);
    if (eq7 === 1'b0) $display("PASS assign eq7"); else $display("FAIL assign eq7=%b", eq7);
    if ((int'(r) == 0) === 1'b1) $display("PASS initial"); else $display("FAIL initial");
  end
endmodule
