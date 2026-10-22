package p; localparam int A[2]='{3,5}; endpackage
module top; import p::*; localparam int N=A[1]; int mem[A[1]];
 initial begin $display("T|imports %0d %0d %0d",N,$size(mem),A[1]); $finish; end endmodule
