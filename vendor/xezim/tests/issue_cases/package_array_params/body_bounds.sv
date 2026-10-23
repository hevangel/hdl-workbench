module leaf; localparam int A[4:5]='{17,29}; endmodule
module top; leaf u(); int i=4; initial begin
 $display("T|bodybounds %0d %0d %0d %0d",$left(u.A),$right(u.A),u.A[i],u.A[i+1]); $finish; end endmodule
