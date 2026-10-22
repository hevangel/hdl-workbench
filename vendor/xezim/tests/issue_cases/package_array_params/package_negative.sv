package p; localparam int A[-2:-1]='{17,29}; endpackage
module top; localparam int N=p::A[-1]; int mem[p::A[-1]]; int i=-2;
 initial begin $display("T|negative %0d %0d %0d %0d %0d",$left(p::A),$right(p::A),N,$size(mem),p::A[i+1]); $finish; end endmodule
