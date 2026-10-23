module top;
logic clk=0; always #5 clk=~clk;
int v=0, prev=0, prev2=0, ticks=0, fails=0, actions=0;
logic en=0;
always @(posedge clk) begin prev2<=prev; prev<=v; v<=v+1; ticks<=ticks+1; en<=~en; end
p1: assert property (@(posedge clk) ticks<2 || $past(v,2)==prev2) else fails++;
p2: assert property (@(posedge clk) ticks<1 || $past(v+1)==prev+1) else fails++;
p3: assert property (@(posedge clk) ticks<1 || $past(v,1,1'b1,@(posedge clk))==prev) else fails++;
p4: assert property (@(posedge clk) ticks<4 || $past(v,1,en)==(v%2 ? v-2:v-1)) else fails++;
p5: assert property (@(posedge clk) 1'b1) begin
 actions++;
 if (ticks>=2 && $past(v)!=prev-1) fails++;
 if (ticks>=3 && $past(v+1,2)!=prev2) fails++;
 if (ticks>1 && !$changed(v,@(posedge clk))) fails++;
end
initial begin #100; if(fails==0 && actions==10) $display("PASS sampled neighbors"); else $display("FAIL sampled neighbors %0d %0d",fails,actions); $finish; end
endmodule
