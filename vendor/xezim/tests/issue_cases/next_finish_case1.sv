module top; int q=0; initial begin #5; q<=1; #0; $finish; q++; end final if(q==0) $display("PASS finish barrier"); else $display("FAIL finish barrier %0d",q); endmodule
