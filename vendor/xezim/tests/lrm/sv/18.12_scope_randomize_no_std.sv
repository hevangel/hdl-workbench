// top: t18_12
module t18_12;
  int x, r;
  initial begin
    r = randomize(x) with { x == 17; }; $display("T|a|r=%0d x=%0d", r, x);
    r = std::randomize(x) with { x == 18; }; $display("T|b|r=%0d x=%0d", r, x);
  end
endmodule
