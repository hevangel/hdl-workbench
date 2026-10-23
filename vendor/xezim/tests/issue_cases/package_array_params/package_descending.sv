package p; localparam int A[5:4]='{17,29}; endpackage
module top; int i=5; initial begin
 $display("T|bounds %0d %0d %0d", $left(p::A),$right(p::A),$size(p::A));
 $display("T|values %0d %0d",p::A[i],p::A[i-1]); $finish; end endmodule
