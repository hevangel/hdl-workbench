module top;
logic [3:0] x, z=4'bz, v=4'bx1z0;
typedef bit [7:0] bits_t; typedef bits_t alias_t;
logic a,b,c,d,e,f,g;
assign a=(int'(x) === 0); assign b=(byte'(z) === 0);
assign c=(shortint'(v) === 4); assign d=(longint'(v) === 4);
assign e=(integer'(x) === 32'h0000000x); assign f=(alias_t'(v) === 8'd4);
assign g=(bit'(z) === 0);
initial #1 begin
 if (a && int'(x)===0) $display("PASS int cast"); else $display("FAIL int cast");
 if (b && byte'(z)===0) $display("PASS byte cast"); else $display("FAIL byte cast");
 if (c && shortint'(v)===4) $display("PASS short cast"); else $display("FAIL short cast");
 if (d && longint'(v)===4) $display("PASS long cast"); else $display("FAIL long cast");
 if (e && integer'(x)===32'h0000000x) $display("PASS four state cast"); else $display("FAIL four state cast");
 if (f && alias_t'(v)===8'd4 && g) $display("PASS typedef and bit cast"); else $display("FAIL typedef and bit cast");
 $finish;
end
endmodule
