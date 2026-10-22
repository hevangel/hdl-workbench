module child_assert_inc(input logic clk); int n=0; a: assert property (@(posedge clk) 1) n++; endmodule
module probe_assert_inc; logic clk=0; always #5 clk=~clk; int n=0;
 child_assert_inc u(.clk(clk)); a: assert property (@(posedge clk) 1) n++;
 initial begin repeat(3) @(posedge clk); #1; $display("T|assert_inc|%0d %0d",n,u.n); end
endmodule
module child_assert_assign(input logic clk); int n=0; a: assert property (@(posedge clk) 1) n=n+1; endmodule
module probe_assert_assign; logic clk=0; always #5 clk=~clk; int n=0;
 child_assert_assign u(.clk(clk)); a: assert property (@(posedge clk) 1) n=n+1;
 initial begin repeat(3) @(posedge clk); #1; $display("T|assert_assign|%0d %0d",n,u.n); end
endmodule
module child_assert_compound(input logic clk); int n=0;  assert property (@(posedge clk) 1) n+=1; endmodule
module probe_assert_compound; logic clk=0; always #5 clk=~clk; int n=0;
 child_assert_compound u(.clk(clk));  assert property (@(posedge clk) 1) n+=1;
 initial begin repeat(3) @(posedge clk); #1; $display("T|assert_compound|%0d %0d",n,u.n); end
endmodule
module child_cover_inc(input logic clk); int n=0; a: cover property (@(posedge clk) 1) n++; endmodule
module probe_cover_inc; logic clk=0; always #5 clk=~clk; int n=0;
 child_cover_inc u(.clk(clk)); a: cover property (@(posedge clk) 1) n++;
 initial begin repeat(3) @(posedge clk); #1; $display("T|cover_inc|%0d %0d",n,u.n); end
endmodule
module child_fail_inc(input logic clk); int n=0; a: assert property (@(posedge clk) 0) else n++; endmodule
module probe_fail_inc; logic clk=0; always #5 clk=~clk; int n=0;
 child_fail_inc u(.clk(clk)); a: assert property (@(posedge clk) 0) else n++;
 initial begin repeat(3) @(posedge clk); #1; $display("T|fail_inc|%0d %0d",n,u.n); end
endmodule
module child_child_idle(input logic clk); int n=0;  endmodule
module probe_child_idle; logic clk=0; always #5 clk=~clk; int n=0;
 child_child_idle u(.clk(clk));  assert property (@(posedge clk) 1) n++;
 initial begin repeat(3) @(posedge clk); #1; $display("T|child_idle|%0d %0d",n,u.n); end
endmodule
module child_block_shadow(input logic clk); int n=0;  assert property (@(posedge clk) 1) begin int n; n=7; end endmodule
module probe_block_shadow; logic clk=0; always #5 clk=~clk; int n=0;
 child_block_shadow u(.clk(clk));  assert property (@(posedge clk) 1) begin int n; n=9; end
 initial begin repeat(3) @(posedge clk); #1; $display("T|block_shadow|%0d %0d",n,u.n); end
endmodule
module child_nested(input logic clk);
 int n=0; child_assert_inc leaf(.clk(clk));
 assert property (@(posedge clk) 1) n++;
endmodule
module probe_nested;
 logic clk=0; always #5 clk=~clk; int n=0; child_nested u(.clk(clk));
 assert property (@(posedge clk) 1) n++;
 initial begin repeat(3) @(posedge clk); #1; $display("T|nested|%0d %0d %0d",n,u.n,u.leaf.n); end
endmodule
module probe_siblings;
 logic clk=0; always #5 clk=~clk; int n=0;
 child_assert_inc a(.clk(clk)),b(.clk(clk));
 assert property (@(posedge clk) 1) n++;
 initial begin repeat(3) @(posedge clk); #1; $display("T|siblings|%0d %0d %0d",n,a.n,b.n); end
endmodule
module probe_generate;
 logic clk=0; always #5 clk=~clk; int n=0;
 for(genvar i=0;i<2;i++)begin:g int n=0; assert property (@(posedge clk) 1) n+=i+1; end
 assert property (@(posedge clk) 1) n++;
 initial begin repeat(3) @(posedge clk); #1; $display("T|generate|%0d %0d %0d",n,g[0].n,g[1].n); end
endmodule
module top;
probe_assert_inc i_assert_inc();
probe_assert_assign i_assert_assign();
probe_assert_compound i_assert_compound();
probe_cover_inc i_cover_inc();
probe_fail_inc i_fail_inc();
probe_child_idle i_child_idle();
probe_block_shadow i_block_shadow();
probe_nested i_nested();
probe_siblings i_siblings();
probe_generate i_generate();
initial #27 $finish; endmodule
