package p; localparam int A[2][2]='{'{3,5},'{7,11}}; endpackage
module top; localparam int N=p::A[0][1]; int i=0,j=1;
 initial begin $display("T|multi %0d %0d %0d",N,p::A[0][1],p::A[i][j]); $finish; end endmodule
