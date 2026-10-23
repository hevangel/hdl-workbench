module top; int q=0; initial begin #5; fork begin q<=1; $finish; end begin #0; q=2; end join end final if(q==0) $display("PASS finish barrier"); else $display("FAIL finish barrier %0d",q); endmodule
