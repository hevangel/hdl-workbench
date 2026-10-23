// top: rsb
module rsb;
  initial begin
    for (int k = 0; k < 3; k++) begin
      static int s = 10;
      s++;
      $display("T|r1|%0d", s);
    end
  end
  task automatic t(); static int c = 0; c++; $display("T|r2|%0d", c); endtask
  initial #1 begin t(); t(); end
endmodule
