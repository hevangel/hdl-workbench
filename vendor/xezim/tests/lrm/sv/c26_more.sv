// top: c26x
package base_p; int shared_v = 1; typedef int my_int; function int f(); return 10; endfunction endpackage
package mid_p; import base_p::*; export base_p::f; int mid_v = 2; endpackage
package top_p; import mid_p::*; export mid_p::*; int top_v = f() - 7; endpackage
// local declaration overrides a wildcard import
module c26x;
  import base_p::*;
  import top_p::*;
  int shared_v = 99;      // local wins over wildcard-imported name
  initial begin
    #1;
    $display("T|26.3e|%0d %0d", shared_v, base_p::shared_v);
    $display("T|26.4|%0d %0d", f(), top_v);
    $display("T|26.3f|%0d", $bits(my_int));
  end
endmodule
