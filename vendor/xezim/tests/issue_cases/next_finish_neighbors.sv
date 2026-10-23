module top;
logic clk=0; int q=0,n=0,after_call=0;
always #5 clk=~clk;
function automatic void stop_now(); $finish; after_call++; endfunction
always @(posedge clk) begin
 n++;
 if(n==3) begin q<=99; stop_now(); after_call++; end
 else q<=q+1;
end
final begin
 if(q==2 && n==3 && after_call==0) $display("PASS nested function finish"); else $display("FAIL nested function finish %0d %0d %0d",q,n,after_call);
end
endmodule
