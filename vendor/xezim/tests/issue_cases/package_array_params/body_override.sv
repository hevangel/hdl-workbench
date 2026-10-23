module leaf; parameter int A[2]='{1,2}; endmodule
module top; leaf u_default(); leaf #(.A('{7,9})) u_override();
 initial begin $display("T|override %0d %0d %0d %0d",$size(u_default.A),$size(u_override.A),u_default.A[1],u_override.A[1]); $finish; end endmodule
