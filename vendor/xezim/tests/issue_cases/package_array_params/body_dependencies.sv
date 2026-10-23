module leaf; localparam int A[2]='{3,5}; localparam int N=A[1]+2; logic[N-1:0] q; endmodule
module top; leaf u(); initial begin $display("T|bodydep %0d %0d %0d",u.A[1],u.N,$bits(u.q)); $finish; end endmodule
