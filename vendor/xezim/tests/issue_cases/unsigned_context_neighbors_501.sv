module top;
 logic signed [7:0] a=-10,b=3;
 logic [7:0] u=100;
 logic [15:0] u16=1000;
 logic choose=0;
 logic signed [7:0] signed_elements[2];
 logic [7:0] unsigned_elements[2];
 logic [1:0] element_index=0;
 wire compiled_array_cmp=(a/b)>signed_elements[element_index];
 wire [7:0] compiled_array_sum=unsigned_elements[element_index]+a/b;
 int calls=0, errors=0;
 function automatic logic signed [7:0] get_a(); calls++; return a; endfunction
 function automatic logic signed [7:0] own_div(); return a/b; endfunction
 task automatic ck(string label,logic [127:0] got,logic [127:0] want);
  if(got !== want) begin errors++; $display("FAIL %s got=%h want=%h",label,got,want);end
 endtask
 wire [7:0] compiled_div=choose ? 8'd0 : a/b;
 wire compiled_cmp=(a/b)>u;
 wire [15:0] compiled_sum=u16+(a+b);
 initial begin
  signed_elements[0]=0;unsigned_elements[0]=100;
  #1;
  ck("compiled signed array element",compiled_array_cmp,0);
  ck("compiled unsigned array element",compiled_array_sum,182);
  ck("compiled division",compiled_div,82);
  ck("compiled comparison",compiled_cmp,0);
  ck("compiled extension",compiled_sum,1249);
  begin logic [7:0] v;
   v=choose ? u : a/b; ck("variable arm",v,82);
   v=choose ? '1 : a/b; ck("ones arm",v,82);
   v=choose ? 'x : a/b; ck("unknown arm",v,82);
   v=choose ? 'z : a%b; ck("high impedance arm",v,0);
   v=unsigned_elements[0]+a/b;ck("unsigned array element",v,182);
   v=unsigned_elements[element_index]+a/b;ck("dynamic unsigned array element",v,182);
   v=u+(a/b);ck("nested div",v,182);
   v=u+(a%b);ck("nested mod",v,100);
   v=choose ? 8'd0 : -(a/b);ck("nested unary",v,8'hae);
   v=choose ? 8'sd0 : a/b;ck("signed arms",v,8'hfd);
  end
  begin logic [15:0] v;
   v=u16+(a+b);ck("add extension",v,1249);
   v=u16+(a-b);ck("subtract extension",v,1243);
   v=u16+(a*b);ck("multiply extension",v,1738);
   v=u16+(a&8'shff);ck("bitwise extension",v,1246);
   v=u16+(a|b);ck("or extension",v,1247);
   v=u16+(a^b);ck("xor extension",v,1245);
   v=u16+(a~^b);ck("xnor extension",v,16'h02f2);
   v=choose ? 16'd0 : (get_a()/b);ck("signed return in expression",v,82);
   ck("call once",calls,1);
   v=u16+own_div();ck("function boundary",v,1253);
   v=u16+$signed(a/b);ck("cast boundary",v,1253);
   v=u16+8'(a/b);ck("size cast boundary",v,1253);
   v={u,a/b};ck("concatenation boundary",v,16'h64fd);
   v={2{a/b}};ck("replication boundary",v,16'hfdfd);
  end
  begin logic v;
   v=(a/b)>signed_elements[0];ck("signed array element",v,0);
   v=(a/b)>signed_elements[element_index];ck("dynamic signed array element",v,0);
   v=(a/b)>u;ck("greater",v,0);
   v=(a/b)<u;ck("less",v,1);
   v=(a/b)==8'd82;ck("equal",v,1);
   v=(a/b)===8'd82;ck("case equal",v,1);
   v=(a/b)!=8'd82;ck("not equal",v,0);
   v=(a/b)==?8'b0101_00??;ck("wildcard",v,1);
   v=(a/b)&&1'b1;ck("logical boundary",v,1);
   v=^(a/b);ck("reduction boundary",v,1);
  end
  begin int x=-5;logic v;logic signed[7:0] sx=-10;logic[7:0] result;
   v=(++x<0);ck("signed preincrement",v,1);
   v=(x--<0);ck("signed postdecrement",v,1);
   v=(--x<0);ck("signed predecrement",v,1);
   v=(x++<0);ck("signed postincrement",v,1);
   result=8'd0+(++sx/b);ck("increment in unsigned division",result,82);
   ck("increment evaluates once",sx,-9);
   v=(64'hffffffffffffffff===8'shff);ck("signed literal compatibility",v,1);
  end
  begin logic signed [127:0] wa=-10,wb=3;logic [127:0] v;
   v=choose ? 128'd0 : wa/wb;
   ck("wide division",v,128'h55555555555555555555555555555552);
   v=choose ? 128'd0 : wa%wb;ck("wide modulus",v,0);
  end
  begin logic [7:0] v;
   v=1'bx ? 8'd82 : a/b;ck("unknown condition merge",v,82);
   v=1'bx ? '0 : a/b;ck("fill merge",v,8'b0x0x00x0);
  end
  if(errors==0)$display("ALL PASS");else $display("FAILED %0d",errors);
 end
endmodule
