// §18.4: `size() inside {…}` redraws a size inside the set.
module b295d;
  bit [15:0] u16 [];
  int r;
  initial begin
    u16 = new[2];
    r = std::randomize(u16) with { u16.size() inside {[3:5]}; };
    $display("T|d1 r=%0d n=%0d", r, u16.size());
    u16 = new[2];
    r = std::randomize(u16) with { u16.size() >= 3; u16.size() <= 5; };
    $display("T|d2 r=%0d n=%0d", r, u16.size());
    u16 = new[2];
    r = std::randomize(u16) with { u16.size() inside {[3:5]}; u16.sum() with (int'(item)) == 70000; };
    $display("T|d3 r=%0d n=%0d", r, u16.size());
    u16 = new[2];
    r = std::randomize(u16) with { u16.size() >= 3; u16.size() <= 5; u16.sum() with (int'(item)) == 70000; };
    $display("T|d4 r=%0d n=%0d", r, u16.size());
  end
endmodule
