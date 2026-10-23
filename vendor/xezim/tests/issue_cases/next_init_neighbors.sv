package p;
logic [7:0] values[3:1]='{2:9, default:7};
endpackage
module child;
logic[7:0] a[3:1]='{2:9,default:7};
initial #1 if (a[3]===7 && a[2]===9 && a[1]===7) $display("PASS child default"); else $display("FAIL child default");
endmodule
module top;
logic[7:0] a[3:1]='{default:7};
logic[7:0] b[-2:0]='{-1:9,default:7};
logic[7:0] d[2:1][0:1]='{default:7};
logic[7:0] e[1:0]='{11,12};
child u();
initial #1 begin
 if (a[3]===7 && a[2]===7 && a[1]===7) $display("PASS descending default"); else $display("FAIL descending default");
 if (b[-2]===7 && b[-1]===9 && b[0]===7) $display("PASS keyed default"); else $display("FAIL keyed default");
 if (d[2][0]===7 && d[2][1]===7 && d[1][0]===7 && d[1][1]===7) $display("PASS nested default"); else $display("FAIL nested default");
 if (e[1]===11 && e[0]===12) $display("PASS descending positional"); else $display("FAIL descending positional");
 if (p::values[3]===7 && p::values[2]===9 && p::values[1]===7) $display("PASS package default"); else $display("FAIL package default");
end
endmodule
