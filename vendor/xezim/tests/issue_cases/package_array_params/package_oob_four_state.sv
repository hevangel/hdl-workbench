package p; localparam integer A[1]='{17}; endpackage
module top; localparam integer A[2]='{91,93}; localparam integer N=p::A[1];
 initial begin $display("T|oob %0d %0d %0d",$isunknown(N),$isunknown(p::A[1]),p::A[0]); $finish; end endmodule
