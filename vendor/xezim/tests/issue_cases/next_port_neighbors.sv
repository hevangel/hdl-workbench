module chk_d(input logic[7:0] p[5:4]);
initial #1 if (p[5]===10 && p[4]===11) $display("PASS descending formal"); else $display("FAIL descending formal %p",p);
endmodule
module chk_n(input logic[7:0] p[-3:-2]);
initial #1 if (p[-3]===21 && p[-2]===22) $display("PASS negative formal"); else $display("FAIL negative formal %p",p);
endmodule
module chk_s(input logic[7:0] p[0:1]);
initial #1 if (p[0]===12 && p[1]===13) $display("PASS sliced actual"); else $display("FAIL sliced actual %p",p);
endmodule
module chk_o(output logic[7:0] p[5:4]); initial begin p[5]=31; p[4]=32; end endmodule
module chk_m(input logic[7:0] p[5:4][-1:0]);
initial #1 if (p[5][-1]===41 && p[5][0]===42 && p[4][-1]===43 && p[4][0]===44) $display("PASS multidimensional mapping"); else $display("FAIL multidimensional mapping %p",p);
endmodule
module chk_old(p); input logic[7:0] p[5:4];
initial #1 if (p[5]===10 && p[4]===11) $display("PASS non ANSI mapping"); else $display("FAIL non ANSI mapping %p",p);
endmodule
module top;
logic[7:0] a[0:3]='{10,11,12,13}; logic[7:0] b[4:3]='{21,22};
logic[7:0] o[10:11]; logic[7:0] m[0:1][3:2]='{'{41,42},'{43,44}};
chk_d d(a[0:1]); chk_n n(b); chk_s s(a[2:3]); chk_o z(o); chk_m mm(m); chk_old old(a[0:1]);
initial #2 if(o[10]===31 && o[11]===32) $display("PASS output mapping"); else $display("FAIL output mapping %p",o);
endmodule
