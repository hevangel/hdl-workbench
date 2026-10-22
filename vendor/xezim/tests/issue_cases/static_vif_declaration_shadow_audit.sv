interface vif_if; int value; endinterface
class Base; static virtual vif_if v; endclass
class Child extends Base;
  virtual vif_if v;
  function void set(virtual vif_if src); this.v=src; endfunction
endclass
class IntChild extends Base; int v=9; endclass
class ParamChild extends Base; localparam int v=23; endclass
class InstanceBase; virtual vif_if v; endclass
class StaticChild extends InstanceBase; static virtual vif_if v; endclass
class Reader;
  static function bit child(Child c, virtual vif_if expected); return c.v==expected; endfunction
  static function bit integer_member(IntChild i); return i.v==9; endfunction
  static function bit parameter_member(ParamChild p); return p.v==23; endfunction
endclass
module top;
  vif_if m(), n(); Child c; IntChild i; ParamChild p;
  InstanceBase b; StaticChild d; virtual vif_if copy;
  initial begin
    m.value=17; n.value=29; c=new; i=new; p=new; b=new; d=new;
    Base::v=m; c.v=n;
    $display("T|parent-after-child %0d",Base::v.value);
    copy=c.v;
    $display("T|copy %0d",copy.value);
    $display("T|child %0d %0d %0d",c.v.value,c.v==n,c.v==Base::v);
    $display("T|scalars %0d %0d %0d %0d",i.v,i.v==9,p.v,p.v==23);
    $display("T|methods %0d %0d %0d",Reader::child(c,n),Reader::integer_member(i),Reader::parameter_member(p));
    c.v=m; c.set(n);
    $display("T|setter %0d %0d",Base::v.value,c.v.value);
    c.v=null;
    $display("T|child-null %0d %0d",c.v==null,Base::v.value);
    b.v=m; StaticChild::v=n;
    $display("T|static-child %0d %0d %0d",b.v.value,d.v.value,d.v==n);
    $finish;
  end
endmodule
