// top: c8_iface4
interface class IA; pure virtual function int f(); typedef int a_t; endclass
interface class IB; pure virtual function int f(); endclass   // same name in two ifaces
interface class IFwd; pure virtual function int g(); endclass
virtual class Part implements IA, IB, IFwd;   // partial: f implemented, g left pure
  virtual function int f(); return 11; endfunction
  pure virtual function int g();
endclass
class Full extends Part; virtual function int g(); return 22; endfunction endclass
class UsesType implements IA; IA::a_t v = 5; virtual function int f(); return v; endfunction endclass
module c8_iface4;
  Full fu; IA ia; IB ib; IFwd iw; UsesType ut;
  initial begin
    fu = new; ia = fu; ib = fu; iw = fu;
    $display("T|8.26.7|partial f=%0d/%0d g=%0d", ia.f(), ib.f(), iw.g());
    ut = new; $display("T|8.26.5|type access %0d", ut.f());
    ia = null; $display("T|8.26|null iface handle %0d", ia == null);
  end
endmodule
