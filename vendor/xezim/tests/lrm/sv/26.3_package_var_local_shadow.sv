// top: rpl
package base_p; int shared_v = 1; endpackage
module rpl;
  import base_p::*;
  int shared_v = 99;
  initial #1 $display("T|r1|local=%0d pkg=%0d", shared_v, base_p::shared_v);
endmodule
