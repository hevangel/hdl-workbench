package p; localparam int A[2]='{3,5}; localparam int B=A[1]+2; localparam int C=p::A[1]+2; endpackage
module top; initial begin $display("T|dependencies %0d %0d %0d",p::A[1],p::B,p::C); $finish; end endmodule
