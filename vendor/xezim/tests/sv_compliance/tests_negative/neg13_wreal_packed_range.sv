// EXPECT: compile_fail
//
// Verilog-AMS 2.4 §3.8: a `wreal` net carries one REAL value, not a vector of
// bits, so it has no packed range. Accepting one is not harmless -- the net
// then behaves as an ordinary 4-bit wire and silently ROUNDS every value
// written to it (2.5 reads back 3.0), which is the exact corruption `wreal`
// exists to prevent, with nothing reported. Both spellings below were once
// accepted in silence; the port form reaches the check by a different path
// than the declaration form, so both are pinned here.
module neg13_wreal_packed_range (input wreal [3:0] p);
  wreal [3:0] w;
  assign w = 2.5;
endmodule
