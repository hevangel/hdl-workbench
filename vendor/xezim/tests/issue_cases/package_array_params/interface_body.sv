interface bus_if; localparam int A[3]='{7,8,9}; endinterface
module top; bus_if b(); initial begin $display("T|interface %0d %0d",$size(b.A),b.A[1]); $finish; end endmodule
