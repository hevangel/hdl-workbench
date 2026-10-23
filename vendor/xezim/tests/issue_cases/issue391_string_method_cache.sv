class copier;
function automatic string copy(string s);
 string out = ""; byte unsigned c;
 for (int i=0; i<s.len(); i++) begin c=s.getc(i); out={out,string'(c)}; end
 return out;
endfunction
endclass
module top;
 copier obj; string result;
 initial begin
  obj=new;
  repeat (4) result=obj.copy("HELLO");
  $display("T|cache|%0d %0d",result=="HELLO",result.len());
  $finish;
 end
endmodule
