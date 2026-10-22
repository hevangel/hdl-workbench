module top;
  logic [63:0] a;
  logic [63:0] one, nested, two, three, four;
  logic [62:0] a63, out63;
  logic [64:0] a65, out65;
  logic [127:0] doubled;
  always_comb one={a};
  always_comb begin
    nested={{a}};
    doubled={a,a};
  end
  always_comb two={a[63:32],a[31:0]};
  always_comb three={a[63:56],a[55:32],a[31:0]};
  always_comb four={a[63:48],a[47:32],a[31:16],a[15:0]};
  always_comb out63={a63};
  always_comb out65={a65};
  initial begin
    a=64'hx123_4567_89ab_cdez; a63=63'hx123_4567_89ab_cdef; a65=65'h1_0123_4567_89ab_cdez;
    #1;
    $display("T|unknown|%h %h %h %h %h",one,nested,two,three,four);
    $display("T|boundaries|%h %h %h",out63,out65,doubled);
    a=64'hfedc_ba98_7654_3210; a63=63'h7123_4567_89ab_cdef; a65=65'h1_0123_4567_89ab_cdef;
    #1;
    $display("T|known|%h %h %h %h %h",one,nested,two,three,four);
    $display("T|known_boundaries|%h %h %h",out63,out65,doubled);
    $finish;
  end
endmodule
