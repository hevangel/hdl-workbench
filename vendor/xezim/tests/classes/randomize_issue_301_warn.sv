// A randomize() that returns 0 is reported on stderr (#301): the class, its
// constraint blocks and where they are; a class failing in a loop is
// reported three times. Here `lo` and `hi` have no common solution.
module issue_301_warn;
  class c1; rand bit [7:0] x; constraint lo { x < 5; } constraint hi { x > 10; } endclass
  class c2; rand bit [7:0] y; constraint ok_c { y < 5; } endclass
  initial begin
    c1 o = new(); c2 p = new(); int r;
    repeat (5) begin r = o.randomize(); $display("T|r=%0d", r); end
    r = p.randomize(); $display("T|p=%0d", r);
    r = p.randomize() with { y == 9; }; $display("T|pw=%0d", r);
  end
endmodule
