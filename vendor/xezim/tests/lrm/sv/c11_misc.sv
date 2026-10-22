// top: c11m
`timescale 1ns/1ns
module c11m;
  logic a = 0;
  wire #(1:2:3) w = a;        // 11.11 min:typ:max -> typ by default
  logic [7:0] x;
  typedef union tagged { void None; int Some; } opt_t;
  opt_t o;
  initial begin
    a = 1; #1 $display("T|11.11a|w=%b", w); #1 $display("T|11.11b|w=%b", w);
    x = (1:2:3); $display("T|11.11c|%0d", x);
    // 11.9 tagged union expression
    o = tagged Some (5);
    $display("T|11.9|%0d", o.Some);
    o = tagged None;
    if (o matches tagged None) $display("T|11.9b|none");
    // 11.10.1 string literals as operands of integral ops
    x = "A" + 1; $display("T|11.10.1|%c", x);
    $display("T|11.10.2|%0d", "ab" == 16'h6162);
  end
endmodule
