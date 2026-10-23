module top;
typedef struct packed {logic [3:0] a; logic [1:0] b;} inner_t;
typedef struct packed {inner_t inner; logic c;} outer_t;
outer_t s='0; logic clk=0; int k=0, fails=0, hits=0;
always #5 clk=~clk;
always @(posedge clk) begin k<=k+1; s.inner.a<=s.inner.a+1; s.inner.b<=s.inner.b+1; s.c<=~s.c; end
p1: assert property (@(posedge clk) s.inner.a==(k%16)) hits++; else fails++;
p2: assert property (@(posedge clk) s.inner.b==(k%4)) hits++; else fails++;
p3: assert property (@(posedge clk) s.c==(k%2)) hits++; else fails++;
p4: assert property (@(posedge clk) s.inner.a[0]==(k%2)) hits++; else fails++;
p5: assert property (@(posedge clk) k<1 || $past(s.inner.a)==((k-1)%16)) hits++; else fails++;
initial begin #100; if(fails==0 && hits==50) $display("PASS preponed neighbors"); else $display("FAIL preponed neighbors %0d %0d",fails,hits); $finish; end
endmodule
