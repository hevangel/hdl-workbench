// §6.16 / §6.24.3 / §11.4.12: byte content and string concat typing.
typedef string text_t;
int calls = 0;
function automatic string convert(logic [31:0] bits); return string'(bits); endfunction
function automatic string alias_convert(logic [31:0] bits); return text_t'(bits); endfunction
function automatic string append(string base, byte unsigned c); return {base, string'(c)}; endfunction
function automatic string local_join(string left, string right); string value; value = {left, right}; return value; endfunction
function automatic string parenthesized(string base, byte unsigned c); return {(base), (string'(c))}; endfunction
function automatic string conditional_join(string base, bit select_right); return {base, select_right ? string'(8'h42) : string'(8'h41)}; endfunction
function automatic logic [31:0] next_word(); calls++; return 32'h41004200; endfunction
function automatic string cast_once(); return string'(next_word()); endfunction
class cast_base;
 typedef string word_text;
endclass
class cast_child extends cast_base;
 function string convert(logic [31:0] bits); return word_text'(bits); endfunction
endclass
module top;
 string a, b, c, d, e, f, g, h, raw;
 cast_child obj;
 initial begin
  obj=new;
  a=convert(32'h41004200); b=convert(0); c=alias_convert(32'h41004200);
  d=append("A",8'he9); e=local_join("AB","CD"); f=parenthesized("A",8'h42);
  g=conditional_join("A",1); h=cast_once(); raw=obj.convert(32'h41004200);
  $display("T|nul|%0d %0d %0d %0d",a=="AB",a.len(),b.len(),c=="AB");
  $display("T|high|%0d %0d",d.len(),$unsigned(d.getc(1)));
  $display("T|concat|%0d %0d %0d",e=="ABCD",f=="AB",g=="AB");
  $display("T|once|%0d %0d %0d",calls,h=="AB",h.len());
  $display("T|class|%0d %0d",raw=="AB",raw.len());
  $finish;
 end
endmodule
