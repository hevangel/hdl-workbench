module probe_x_rise;
 logic clk;
 initial begin clk=1; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|x_rise|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_zero_rise;
 logic clk=0;
 initial begin clk=1; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|zero_rise|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_x_fall;
 logic clk;
 initial begin clk=0; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(negedge clk) seen++;
 c: cover property (@(negedge clk) 1) covers++;
 a: assert property (@(negedge clk) 1) passes++;
 initial #31 $display("T|x_fall|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_one_fall;
 logic clk=1;
 initial begin clk=0; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(negedge clk) seen++;
 c: cover property (@(negedge clk) 1) covers++;
 a: assert property (@(negedge clk) 1) passes++;
 initial #31 $display("T|one_fall|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_late_rise;
 logic clk=0;
 initial begin #3 clk=1; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|late_rise|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_declared_one;
 logic clk=1;
 always #5 clk=~clk;
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|declared_one|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_declared_zero;
 logic clk=0;
 always #5 clk=~clk;
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|declared_zero|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_zero_to_x;
 logic clk=0;
 initial begin clk='x; #5 clk=0; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|zero_to_x|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_one_to_z;
 logic clk=1;
 initial begin clk='z; #5 clk=1; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(negedge clk) seen++;
 c: cover property (@(negedge clk) 1) covers++;
 a: assert property (@(negedge clk) 1) passes++;
 initial #31 $display("T|one_to_z|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_x_to_z;
 logic clk;
 initial begin clk='z; #5 clk=0; forever #5 clk=~clk; end
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|x_to_z|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_vector_lsb;
 logic [3:0] clk=0;
 initial begin clk=8; #5 clk=9; #5 clk=1; #5 clk=0; #5 clk=8; #5 clk=9; #5 clk=1; end
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|vector_lsb|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_vector_x_msb;
 logic [3:0] clk=4'bx000;
 initial begin clk[0]=1; forever #5 clk[0]=~clk[0]; end
 int seen=0,covers=0,passes=0;
 always @(posedge clk) seen++;
 c: cover property (@(posedge clk) 1) covers++;
 a: assert property (@(posedge clk) 1) passes++;
 initial #31 $display("T|vector_x_msb|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_iff_initial;
 logic clk=0; logic en=0;
 initial begin clk=1; forever #5 clk=~clk; end initial #1 en=1;
 int seen=0,covers=0,passes=0;
 always @(posedge clk iff en) seen++;
 c: cover property (@(posedge clk iff en) 1) covers++;
 a: assert property (@(posedge clk iff en) 1) passes++;
 initial #31 $display("T|iff_initial|%0d %0d %0d",seen,covers,passes);
endmodule
module probe_preponed;
 logic clk=0; bit data=0;
 int passes=0,fails=0;
 initial begin data=1; clk=1; forever #5 clk=~clk; end
 a: assert property (@(posedge clk) data==0) passes++; else fails++;
 initial #31 $display("T|preponed|%0d %0d",passes,fails);
endmodule
module top;
probe_x_rise i_x_rise();
probe_zero_rise i_zero_rise();
probe_x_fall i_x_fall();
probe_one_fall i_one_fall();
probe_late_rise i_late_rise();
probe_declared_one i_declared_one();
probe_declared_zero i_declared_zero();
probe_zero_to_x i_zero_to_x();
probe_one_to_z i_one_to_z();
probe_x_to_z i_x_to_z();
probe_vector_lsb i_vector_lsb();
probe_vector_x_msb i_vector_x_msb();
probe_iff_initial i_iff_initial();
probe_preponed i_preponed(); initial #32 $finish; endmodule
