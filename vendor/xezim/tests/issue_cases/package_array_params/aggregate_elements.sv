// Aggregate parameter elements retain whole values (§6.20.2 / §10.9.2).
module top;
 typedef struct { int s; bit [1:0] x; } u_t;
 typedef u_t alias_t;
 typedef struct packed { bit [31:0] s; bit [1:0] x; } p_t;
 localparam u_t LU[2]='{'{9,1},'{10,2}};
 parameter u_t PU[2]='{'{7,1},'{8,2}};
 localparam alias_t AL[1]='{'{13,3}};
 localparam p_t LP[2]='{'{4,1},'{5,2}};
 localparam bit [2:0] BV[2]='{'{1,0,1},'{0,1,1}};
 localparam int I[1]='{17};
 localparam integer J[1]='{19};
 initial begin
  $display("T|aggregate %0d %0d %0d %0d %0d %0d %0d %b %b %0d %0d",LU[0].s,LU[1].s,PU[0].s,PU[1].s,AL[0].s,LP[0].s,LP[1].s,BV[0],BV[1],$isunknown(I[1]),$isunknown(J[1]));
  $finish;
 end
endmodule
