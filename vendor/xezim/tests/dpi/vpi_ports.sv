// `vpiPort` objects and `vpiDirection` (IEEE 1800-2017 §37.16): a port is
// its own object, iterated from a module; the connected net or variable
// keeps its `vpiNet`/`vpiReg` type. Probed on the top module and on a
// sub-instance (whose ports are recreated by the inliner).
module tb(input logic clk_in, output logic [3:0] o_top, inout wire [1:0] io_top);
  logic [3:0] a = 4'h5; logic [3:0] y; wire w = 1'b1;
  assign o_top = a;
  sub u_sub(.clk(clk_in), .i(a), .o(y), .io(io_top));
  initial begin #1 $ports_probe; #1 $finish; end
endmodule
module sub(input logic clk, input logic [3:0] i, output logic [3:0] o, inout wire [1:0] io);
  logic [3:0] internal; assign o = i; always @(posedge clk) internal <= i;
endmodule
