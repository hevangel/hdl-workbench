module checker_unit(input logic clk);
int k=0, prev=0, errors=0; logic flag=0;
always @(posedge clk) begin prev<=k; k<=k+1; flag<=~flag; end
p1: assert property (@(posedge clk) k<1 || $past(k,1,1'b1,@(posedge clk))==prev) else errors++;
p2: assert property (@(posedge clk) k<1 || $rose(flag,@(posedge clk))==flag) else errors++;
p3: assert property (@(posedge clk) k<1 || $fell(flag,@(posedge clk))==!flag) else errors++;
p4: assert property (@(posedge clk) k<1 || !$stable(k,@(posedge clk))) else errors++;
p5: assert property (@(posedge clk) k<1 || $changed(k,@(posedge clk))) else errors++;
final if(errors==0) $display("PASS explicit clock neighbors"); else $display("FAIL explicit clock neighbors %0d",errors);
endmodule
module top;
logic a=0,b=0; always #3 a=~a; always #5 b=~b;
checker_unit u(a),v(b);
initial #100 $finish;
endmodule
