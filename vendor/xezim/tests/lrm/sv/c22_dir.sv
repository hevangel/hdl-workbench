// top: c22top
`define ADD(a, b) ((a) + (b))
`define DEF(x, y = 5) ((x) * (y))
`define STR(x) `"x`"
`define CAT(a, b) a``b
`define EMPTY
`define MULTI(v) \
  v = v + 1; \
  v = v * 2;
`define NOARGS 42
`define WITHDEF(a=1, b=2) (a*10+b)
`include "c22_inc.svh"
`include "c22_inc.svh"
`timescale 1ns/100ps
module c22;
  int foo_bar = 3;
  int v;
  initial begin
    $display("T|22.5.1a|%0d %0d %0d", `ADD(1, 2), `DEF(3), `DEF(3, 4));
    $display("T|22.5.1b|%s", `STR(hello world));
    $display("T|22.5.1c|%0d", `CAT(foo, _bar));
    $display("T|22.5.1e|%0d", `NOARGS `EMPTY);
    $display("T|22.5.1f|%0d %0d %0d", `WITHDEF(), `WITHDEF(3), `WITHDEF(,4));
    v = 1; `MULTI(v) $display("T|22.5.1g|%0d", v);
    $display("T|22.5.1h|%0d", `ADD(`ADD(1,1), {2,3} == 0 ? 0 : 1 ));
`ifdef NOARGS
    $display("T|22.6a|ifdef ok");
`elsif EMPTY
    $display("T|22.6a|elsif wrong");
`else
    $display("T|22.6a|else wrong");
`endif
`ifndef NOPE
    $display("T|22.6b|ifndef ok");
`endif
`ifdef NOPE
`elsif NOARGS
    $display("T|22.6c|elsif ok");
`endif
`undef NOARGS
`ifdef NOARGS
    $display("T|22.5.2|undef failed");
`else
    $display("T|22.5.2|undef ok");
`endif
    $display("T|22.4|%0d %0d", `INC_VAL, INC_LINE);
    #1.25 $display("T|22.7|%0t %f", $time, $realtime);
    $display("T|22.13|%s %0d", `__FILE__, `__LINE__);
`line 500 "fake.sv" 0
    $display("T|22.12|%s %0d", `__FILE__, `__LINE__);
  end
endmodule
`default_nettype none
module c22b(input wire a, output wire y);
  assign y = a;
endmodule
`default_nettype wire
`resetall
`timescale 1ns/1ns
`celldefine
module c22c; initial $display("T|22.10|celldefine ok"); endmodule
`endcelldefine
`begin_keywords "1364-2005"
module c22d; reg logic; initial begin logic = 1; $display("T|22.14|logic as ident=%b", logic); end endmodule
`end_keywords
`undefineall
`ifdef ADD
module c22e; initial $display("T|22.5.3|undefineall failed"); endmodule
`else
module c22e; initial $display("T|22.5.3|undefineall ok"); endmodule
`endif
module c22top; c22 a(); c22c b(); c22d c(); c22e d(); wire y; c22b e(1'b1, y); initial #2 $display("T|22.8|y=%b", y); endmodule
