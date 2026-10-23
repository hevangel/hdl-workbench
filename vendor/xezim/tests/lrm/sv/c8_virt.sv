// top: c8_virt
virtual class Shape;
  pure virtual function real area();
  virtual function string name(); return "shape"; endfunction
  function string desc(); return $sformatf("%s:%0.1f", name(), area()); endfunction
endclass
class Sq extends Shape;
  real s; function new(real s); this.s = s; endfunction
  virtual function real area(); return s*s; endfunction
  virtual function string name(); return "sq"; endfunction
endclass
class Ci extends Shape;
  virtual function real area(); return 3.0; endfunction
endclass
interface class IPut #(type T = int);
  pure virtual function void put(T x);
  pure virtual function int cnt();
endclass
interface class IGet #(type TG = int);
  pure virtual function TG get();
endclass
interface class IBoth #(type T=int) extends IPut#(T), IGet#(T);
endclass
class Fifo implements IPut#(int), IGet#(int);
  int q[$];
  virtual function void put(int x); q.push_back(x); endfunction
  virtual function int get(); return q.pop_front(); endfunction
  virtual function int cnt(); return q.size(); endfunction
endclass
class F2 implements IBoth#(byte);
  byte q[$];
  virtual function void put(byte x); q.push_back(x); endfunction
  virtual function byte get(); return q.pop_back(); endfunction
  virtual function int cnt(); return q.size(); endfunction
endclass
interface class IConst; parameter int K = 4; endclass
class UsesK implements IConst; function int k(); return IConst::K * 2; endfunction endclass
class Proto;
  extern function int f(int a);
  extern virtual task t(output int o);
  extern static function int sf();
endclass
function int Proto::f(int a); return a * 3; endfunction
task Proto::t(output int o); #1 o = 11; endtask
function int Proto::sf(); return 99; endfunction
typedef class Later;
class Early; Later l; function int g(); l = new; return l.z; endfunction endclass
class Later; int z = 42; endclass
class OuterC;
  static int sv = 3;
  class InnerC; int w = 8; function int gs(); return sv + w; endfunction endclass
  typedef enum {RED, GREEN} col_t;
  function int mk(); InnerC i = new; return i.gs(); endfunction
endclass
class Pfx; static function int f(); return 1; endfunction endclass
module c8_virt;
  Shape s; Sq sq; Ci ci; Fifo f; IPut#(int) ip; IGet#(int) ig; F2 f2; IBoth#(byte) ib; UsesK uk; Proto pr; Early e; OuterC oc;
  int o;
  initial begin
    sq = new(2.0); s = sq; $display("T|8.21|abstract %s", s.desc());
    ci = new; s = ci; $display("T|8.21|abstract2 %s", s.desc());
    f = new; ip = f; ip.put(5); ip.put(6); ig = f; $display("T|8.26|iface get=%0d cnt=%0d", ig.get(), ip.cnt());
    if ($cast(ig, ip)) $display("T|8.26.6|cast between ifaces %0d", ig.get());
    f2 = new; ib = f2; ib.put(8'h7f); ib.put(-1); $display("T|8.26.3|extends ifaces %0d %0d", ib.get(), ib.cnt());
    uk = new; $display("T|8.26.4|iface param %0d %0d", uk.k(), IConst::K);
    pr = new; $display("T|8.24|extern f=%0d sf=%0d", pr.f(4), Proto::sf()); pr.t(o); $display("T|8.24|extern task o=%0d t=%0t", o, $time);
    e = new; $display("T|8.27|typedef class %0d", e.g());
    oc = new; $display("T|8.23|nested %0d %0d", oc.mk(), OuterC::GREEN);
    begin automatic OuterC::InnerC ic = new; $display("T|8.23|nested scope %0d", ic.w); end
    $display("T|8.23|scope res %0d", Pfx::f());
  end
endmodule
