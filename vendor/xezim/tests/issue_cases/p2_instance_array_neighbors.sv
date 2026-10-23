module slice_leaf(input logic [1:0] d, output logic [1:0] q);
  assign q = d;
endmodule
module ansi_wrap(input logic [7:0] d, output logic [7:0] q);
  slice_leaf a[0:1][1:0](.q(q), .d(d));
endmodule
module top;
  logic [7:0] d=8'he4, q0,q1,q2,q3,q4,q5,q6,q7;
  logic [1:0] up[3:2][0:1];
  ansi_wrap w(d,q0);
  slice_leaf a[3:2](.d(d[7:4]),.q(q1[7:4]));
  slice_leaf b[1:0][0:1](.q({q2[7:5],q2[4:0]}),.d(d));
  slice_leaf c[1:0](.d({d[3:2],d[1:0]}),.q(q3[3:0]));
  slice_leaf e[1:0][1:0](.d(d ^ 8'h55),.q(q4));
  slice_leaf f[1:0][1:0](.d(~d),.q(q5));
  slice_leaf g[3:0](.d(1'b1 ? d : 8'h00),.q(q6));
  slice_leaf h[1:0][0:1](.d(up),.q(q7));
  initial begin
    up[3][0]=3; up[3][1]=2; up[2][0]=1; up[2][1]=0;
    #1;
    $display("%s ansi multidimensional", q0===d && w.a[0][1].d===3 && w.a[1][0].d===0 ? "PASS" : "FAIL");
    $display("%s nonzero range", q1[7:4]===d[7:4] && a[3].d===3 && a[2].d===2 ? "PASS" : "FAIL");
    $display("%s crossing concat", q2===d && b[1][0].d===3 && b[0][1].d===0 ? "PASS" : "FAIL");
    $display("%s input concat", q3[3:0]===d[3:0] ? "PASS" : "FAIL");
    $display("%s binary expression", q4===(d^8'h55) ? "PASS" : "FAIL");
    $display("%s unary expression", q5===(~d) ? "PASS" : "FAIL");
    $display("%s conditional expression", q6===d ? "PASS" : "FAIL");
    $display("%s descending unpacked", q7===d ? "PASS" : "FAIL");
    $finish;
  end
endmodule
