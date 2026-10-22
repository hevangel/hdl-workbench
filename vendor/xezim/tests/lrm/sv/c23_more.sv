// top: c23x
module cnt_m #(parameter int START = 0) (input clk, output int q);
  initial q = START;
  always @(posedge clk) q <= q + 1;
  function int peek(); return q; endfunction
  task automatic bump(int n); q = q + n; endtask
endmodule
module wrap #(parameter P = 1);
  cnt_m #(.START(P * 10)) c(.clk(1'b0), .q());
  localparam LP = P + 1;
endmodule
module inout_m(inout wire [3:0] io, input en, input [3:0] val);
  assign io = en ? val : 4'bz;
endmodule
module c23x;
  logic clk = 0;
  int qv;
  cnt_m #(5) u1(clk, qv);
  wrap #(3) w();
  wire [3:0] shared;
  logic e1 = 0, e2 = 0;
  inout_m m1(shared, e1, 4'h3);
  inout_m m2(.io(shared), .en(e2), .val(4'hc));
  // 23.3.2.4 implicit .name with different widths not allowed; positional with expressions
  // 23.6 hierarchical function/task calls
  // 23.9 scope rules: named blocks
  initial begin : named_init
    int local_v;
    local_v = 4;
    #1;
    $display("T|23.6e|%0d %0d", u1.peek(), w.c.peek());
    u1.bump(10); $display("T|23.6f|%0d", u1.q);
    $display("T|23.6g|%0d %0d", w.LP, w.c.START);
    $display("T|23.9a|%0d", c23x.named_init.local_v);
    $display("T|23.6h|%0d", $root.c23x.w.c.q);
    $display("T|23.3.3c|%b", shared);
    e1 = 1; #1 $display("T|23.3.3d|%h", shared);
    e2 = 1; #1 $display("T|23.3.3e|%b", shared);
    // 23.10.4 elaboration-time parameter values
    $display("T|23.10|%0d", $bits(w.c.q));
    clk = 1; #1 $display("T|23.6i|%0d", qv);
  end
endmodule
