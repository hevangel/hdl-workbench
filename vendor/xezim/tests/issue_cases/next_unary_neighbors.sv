module top;
logic [7:0] u=3; logic signed [7:0] s=3;
logic [199:0] w=3; logic signed [199:0] sw=3;
logic a,b,d,e; assign a=(-u<0); assign b=(-s<0); assign d=(-w<0); assign e=(-sw<0);
parameter [7:0] U=3; parameter signed [7:0] S=3;
initial #1 begin
 if (!a && !(-u<0)) $display("PASS unsigned byte"); else $display("FAIL unsigned byte");
 if (b && (-s<0)) $display("PASS signed byte"); else $display("FAIL signed byte");
 if (!d && !(-w<0)) $display("PASS unsigned wide"); else $display("FAIL unsigned wide");
 if (e && (-sw<0)) $display("PASS signed wide"); else $display("FAIL signed wide");
 if (!(-U<0) && (-S<0)) $display("PASS parameter types"); else $display("FAIL parameter types");
 if (-8'bx === 8'bx && -(8'sbx) === 8'bx) $display("PASS unknown negation"); else $display("FAIL unknown negation");
 $finish;
end
endmodule
