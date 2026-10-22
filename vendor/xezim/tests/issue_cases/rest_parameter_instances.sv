`timescale 1ns/1ns
package instance_sizes;
 parameter int COUNT=2;
endpackage
interface parameter_link(input logic clk, input logic [7:0] tag);
endinterface
module parameter_group #(parameter int COUNT=2)(input logic clk);
 parameter_link sized[COUNT](clk,8'hA5);
 initial begin
  #6;
  if($bits(sized[0].clk)==1 && sized[0].clk===1'b1 && sized[COUNT-1].tag===8'hA5)
   $display("PASS parameter group %0d",COUNT);
  else $display("FAIL parameter group %0d",COUNT);
 end
endmodule
module parameter_piece(input logic [7:0] value, output wire [7:0] copied);
 assign copied=value;
endmodule
module top;
 import instance_sizes::*;
 localparam int LOCAL_COUNT=2;
 typedef int index_t;
 int keyed[index_t];
 bit clk;
 wire [15:0] copied;
 always #5 clk=~clk;
 parameter_link imported[COUNT](clk,8'h3C);
 parameter_link scoped_sized[instance_sizes::COUNT](clk,8'h69);
 parameter_link local_sized[LOCAL_COUNT](clk,8'h96);
 parameter_link grid[COUNT][LOCAL_COUNT](clk,8'h5A);
 parameter_group default_group(clk);
 parameter_group #(.COUNT(3)) overridden_group(clk);
 parameter_piece pieces[COUNT](16'hA1B2,copied);
 initial begin
  keyed[7]=9;
  #6;
  if($bits(imported[0].clk)==1 && imported[0].clk===1'b1 && imported[1].tag===8'h3C)
   $display("PASS imported size"); else $display("FAIL imported size");
  if($bits(scoped_sized[0].clk)==1 && scoped_sized[1].tag===8'h69)
   $display("PASS scoped size"); else $display("FAIL scoped size");
  if($bits(local_sized[0].clk)==1 && local_sized[1].tag===8'h96)
   $display("PASS local size"); else $display("FAIL local size");
  if($bits(grid[0][0].clk)==1 && grid[1][1].tag===8'h5A)
   $display("PASS multidimensional size"); else $display("FAIL multidimensional size");
  if(copied===16'hA1B2) $display("PASS module size"); else $display("FAIL module size");
  if(keyed.num()==1 && keyed[7]==9) $display("PASS associative type"); else $display("FAIL associative type");
 end
 initial begin #20; $finish; end
endmodule
