module top;
logic clk=0,en=0; int k=0,failures=0,hits=0;
always #1 clk=~clk;
always @(posedge clk) begin k<=k+1; en<=~en; end
p: assert property (@(posedge clk) 1'b1) begin
 hits++;
 if($past(k)!=(k>=2?k-2:0)) failures++;
 if($past(k,2)!=(k>=3?k-3:0)) failures++;
 if($past(k,17)!=(k>=18?k-18:0)) failures++;
 if($past(k+3)!=(k>=2?k+1:3)) failures++;
 if(k>4 && $past(k,1,en)!=((k-1)%2?k-2:k-3)) failures++;
 if(k>8 && $past(k,3,en)!=((k-1)%2?k-6:k-7)) failures++;
end
initial begin #80; if(failures==0 && hits==40) $display("PASS action neighbors"); else $display("FAIL action neighbors %0d %0d",failures,hits); $finish; end
endmodule
