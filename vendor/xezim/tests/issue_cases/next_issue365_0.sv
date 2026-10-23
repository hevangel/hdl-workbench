module top;
  logic [7:0] a [0:3] = '{default: 7};
  logic [7:0] b [0:3] = '{default: 8'd7};
  logic [7:0] c [0:3] = '{7, 7, 7, 7};
  logic [7:0] d [0:3];
  initial d = '{default: 7};
  initial #1 begin
    $display("a=%0d %0d %0d %0d", a[0], a[1], a[2], a[3]);
    $display("b=%0d %0d %0d %0d", b[0], b[1], b[2], b[3]);
    $display("c=%0d %0d %0d %0d", c[0], c[1], c[2], c[3]);
    $display("d=%0d %0d %0d %0d", d[0], d[1], d[2], d[3]);
  end
endmodule
