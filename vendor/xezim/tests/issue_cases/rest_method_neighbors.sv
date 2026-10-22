typedef logic signed [15:0] byteunsigned;
typedef bit [9:0] timesigned;
// §8.25: specialization types remain distinct after compilation and cache reuse.
class typed_box #(type T = int);
 function longint fitted(longint v); T x; x=v; return x; endfunction
 function longint ones(); T x; x='1; return x; endfunction
endclass
class derived_box #(type U=byte) extends typed_box #(U); endclass
class typed_probe #(type T = byte);
  static function int check(T x);
    T y = x;
    return $bits(y) * 1000 + (y < 0);
  endfunction
endclass
class typed_relay #(type U = byte unsigned);
  static function int check();
    return typed_probe#(U)::check(U'(200));
  endfunction
endclass
module top;
 typedef logic signed [15:0] word_t;
 initial begin
  typed_box #(byte) sb=new;
  typed_box #(byte unsigned) ub=new;
  typed_box #(logic [63:0]) wide_box=new;
  derived_box #(word_t) inherited_box=new;
  typed_box #(logic signed [15:0]) direct_signed=new;
  typed_box #(byteunsigned) named_signed=new;
  typed_box #(timesigned) named_unsigned=new;
  int failures=0;
  for(int k=0;k<2003;k++) begin
   if(named_signed.fitted(65534)!=-2 || named_signed.ones()!=-1) failures++;
   if(named_unsigned.fitted(1023)!=1023 || named_unsigned.ones()!=1023) failures++;
   if(ub.fitted(200)!=200 || sb.fitted(200)!=-56) failures++;
   if(wide_box.ones()!=-1 || ub.ones()!=255 || sb.ones()!=-1) failures++;
   if(inherited_box.fitted(65534)!=-2 || inherited_box.ones()!=-1) failures++;
   if(direct_signed.fitted(65534)!=-2 || direct_signed.ones()!=-1) failures++;
  end
  if(typed_probe#(byte unsigned)::check(200)!=8000) failures++;
  if(typed_relay#(byte unsigned)::check()!=8000) failures++;
  if(typed_probe#(time signed)::check(-1)!=64001) failures++;
  if(typed_probe#(byte)::check(-1)!=8001) failures++;
  if(typed_probe#(shortint unsigned)::check(-1)!=16000) failures++;
  if(typed_probe#(int unsigned)::check(-1)!=32000) failures++;
  if(typed_probe#(byteunsigned)::check(-2)!=16001) failures++;
  if(typed_probe#(timesigned)::check(-1)!=10000) failures++;
  if(typed_relay#(byteunsigned)::check()!=16000) failures++;
  if(failures==0) $display("PASS method neighbors"); else $display("FAIL method neighbors %0d",failures);
 end
endmodule
