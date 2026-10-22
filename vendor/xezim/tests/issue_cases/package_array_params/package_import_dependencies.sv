// Imported array bounds and initializers use the exporting package (§26.3).
package p;
 localparam int L=4, R=5, V=17;
 localparam int A[L:R]='{V,V+12};
endpackage
module top;
 localparam int L=8, R=9, V=91;
 import p::A;
 localparam int N=A[5];
 logic [N-1:0] data;
 initial begin
  $display("T|importenv %0d %0d %0d %0d %0d",$left(A),$right(A),A[4],N,$bits(data));
  $finish;
 end
endmodule
