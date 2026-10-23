// top: t23e
// 23.3.3.5 instance arrays: port slicing and replication
module cell4 #(parameter int P = 1) (input [3:0] a, input b, output [3:0] y, output z);
  assign y = a + P;
  assign z = b ^ (P[0]);
  int id = P;
endmodule
module bitm(input i, output o); assign o = ~i; endmodule
module t23e;
  logic [15:0] bus = 16'h4321;
  logic one = 1;
  wire [15:0] ys; wire [3:0] zs;
  cell4 u[3:0] (.a(bus), .b(one), .y(ys), .z(zs));
  // ascending range
  wire [15:0] ys2; wire [0:3] zs2;
  cell4 #(2) v[0:3] (.a(bus), .b(1'b0), .y(ys2), .z(zs2));
  // single-bit instance array
  logic [7:0] ib = 8'b1010_0110; wire [7:0] ob;
  bitm bb[7:0] (.i(ib), .o(ob));
  // unpacked array actuals, one element per instance
  logic [3:0] ua [4] = '{4'h1, 4'h2, 4'h3, 4'h4};
  wire [3:0] uy [4]; wire [3:0] uz;
  cell4 w[3:0] (.a(ua), .b(1'b1), .y(uy), .z(uz));
  // defparam into an instance array element
  wire [15:0] ys3; wire [3:0] zs3;
  cell4 dpa[1:0] (.a(8'h00), .b(1'b0), .y(ys3[7:0]), .z(zs3[1:0]));
  defparam dpa[1].P = 5;
  // instance array with non-zero-based range
  wire [7:0] oy; wire [1:0] oz;
  cell4 #(.P(3)) nz[5:4] (.a(8'h11), .b(1'b1), .y(oy), .z(oz));
  initial begin
    #1;
    $display("T|23.3.3.5a|ys=%h zs=%b", ys, zs);
    $display("T|23.3.3.5b|ys2=%h zs2=%b", ys2, zs2);
    $display("T|23.3.3.5c|ob=%b", ob);
    $display("T|23.3.3.5d|uy=%p uz=%b", uy, uz);
    $display("T|23.3.3.5e|ys3=%h id1=%0d id0=%0d", ys3[7:0], dpa[1].id, dpa[0].id);
    $display("T|23.3.3.5f|oy=%h oz=%b nz5=%0d nz4.y=%h", oy, oz, nz[5].id, nz[4].y);
    $display("T|23.3.3.5g|u3.a=%h u0.a=%h v0.a=%h v3.a=%h", u[3].a, u[0].a, v[0].a, v[3].a);
    $display("T|23.3.3.5h|%0d", $size(ys));
  end
endmodule
