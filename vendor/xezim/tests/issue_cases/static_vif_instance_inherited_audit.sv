interface vif_if; int value; endinterface
class Base #(type T=int);
  static virtual vif_if v;
  function int read(); return v.value; endfunction
  function bit empty(); return v==null; endfunction
endclass
class Child extends Base#(int); endclass
module top;
  vif_if m(); Base#(int) a; Base#(bit) b; Child c; virtual vif_if copy;
  initial begin
    m.value=17; a=new; b=new; c=new; Base#(int)::v=m;
    $display("T|methods %0d %0d %0d",a.read(),c.read(),b.empty());
    $display("T|members %0d %0d %0d",a.v.value,c.v.value,b.v==null);
    copy=c.v; $display("T|copy %0d",copy.value);
    c.v=null; $display("T|clear %0d %0d",a.empty(),Base#(int)::v==null);
    $finish;
  end
endmodule
