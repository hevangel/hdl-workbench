module top;
logic clk=0; always #1 clk=~clk;
int k=0; always @(posedge clk) k<=k+1;
wire a=(k==1), b=(k>=2 && k<=301), e=(k>=2 && k<=600 && k%2==0);
int r256=0,r257=0,r300=0,r301=0,d4096=0,d4097=0,drange=0,g300=0,n300=0,live=0;
c1: cover property (@(posedge clk) a ##1 b[*256] ##1(k==258)) r256++;
c2: cover property (@(posedge clk) a ##1 b[*257] ##1(k==259)) r257++;
c3: cover property (@(posedge clk) a ##1 b[*300] ##1 !b) r300++;
c4: cover property (@(posedge clk) a ##1 b[*301]) r301++;
c5: cover property (@(posedge clk) a ##4096(k==4097)) d4096++;
c6: cover property (@(posedge clk) a ##4097(k==4098)) d4097++;
c7: cover property (@(posedge clk) a ##[4999:5001](k==5001)) drange++;
c8: cover property (@(posedge clk) a ##1 e[->300] ##1(k==601)) g300++;
c9: cover property (@(posedge clk) a ##1 e[=300] ##1(k==603)) n300++;
c10: cover property (@(posedge clk) (k<1100) ##5000(k==5000)) live++;
initial begin #12010;
 if(r256==1 && r257==1 && r300==1 && r301==0) $display("PASS repetition boundaries"); else $display("FAIL repetition boundaries %0d %0d %0d %0d",r256,r257,r300,r301);
 if(d4096==1 && d4097==1 && drange==1) $display("PASS delay boundaries"); else $display("FAIL delay boundaries %0d %0d %0d",d4096,d4097,drange);
 if(g300==1 && n300==1) $display("PASS goto and nonconsecutive"); else $display("FAIL goto and nonconsecutive %0d %0d",g300,n300);
 if(live==1) $display("PASS live attempt preservation"); else $display("FAIL live attempt preservation %0d",live);
 $finish;
end
endmodule
