package p; localparam int A[1]='{17}; endpackage
module top; localparam int A[2]='{91,93}; localparam int N=p::A[1];
 initial begin $display("T|oob %0d %0d %0d",$isunknown(N),$isunknown(p::A[1]),p::A[0]); $finish; end endmodule
