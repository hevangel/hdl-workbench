// top: c8_basic
class Packet;
  int id; bit [7:0] data; logic [3:0] lg; string name; real r;
  int arr[4]; int q[$]; int aa[string]; byte dyn[];
  static int count = 0;
  static int scount;
  int seq;
  function new(int id = 7, string nm = "dflt");
    this.id = id; name = nm; count++; seq = count;
  endfunction
  static function int get_count(); return count; endfunction
  function int getid(); return id; endfunction
  function Packet self(); return this; endfunction
endclass
class Base;
  int v = 1;
  int a;
  function new(int a); this.a = a; endfunction
  virtual function string who(); return "Base"; endfunction
  function string nv(); return "Base.nv"; endfunction
  virtual function int calc(int x); return x + v; endfunction
endclass
class Derived extends Base;
  int v = 2; // shadows
  function new(int a, int b); super.new(a + b); endfunction
  virtual function string who(); return "Derived"; endfunction
  function string nv(); return "Derived.nv"; endfunction
  virtual function int calc(int x); return super.calc(x) * 10 + v + super.v; endfunction
endclass
class D2 extends Derived;
  function new(); super.new(1, 2); endfunction
  function string who(); return "D2"; endfunction // implicitly virtual
endclass
class ExtArgs extends Base(5);  // 2023/2017: extends with args
endclass
`ifdef NEWDEF
class Defs extends Base;
  function new(default); super.new(default); endfunction
endclass
`endif
class Node; int val; Node next; function new(int v); val = v; endfunction endclass
class Inner; int x = 3; endclass
class Outer; int y = 1; Inner in_h = new; endclass
module c8_basic;
  Packet p, p2, p3; Base b; Derived d, d2; D2 dd; Node n; Outer o1, o2; ExtArgs ea;
  initial begin
    p = new; $display("T|8.7|defaults id=%0d name=%s count=%0d", p.id, p.name, Packet::count);
    $display("T|8.4|uninit data=%h lg=%b name='%s' r=%f arr0=%0d qsz=%0d", p.data, p.lg, p.name, p.r, p.arr[0], p.q.size());
    p2 = new(3, "two"); $display("T|8.7|args id=%0d name=%s count=%0d seq=%0d", p2.id, p2.name, Packet::get_count(), p2.seq);
    $display("T|8.10|static via handle %0d %0d", p.count, p2.get_count());
    p.scount = 5; $display("T|8.9|static shared %0d", p2.scount);
    $display("T|8.11|this %0d", p.self() == p);
    p3 = null; $display("T|8.4|null %0d %0d", p3 == null, p3 != p);
    p.q.push_back(1); p.aa["k"] = 9; p.dyn = new[3]; p.dyn[2] = 8'hff;
    $display("T|8.5|props q=%p aa=%p dyn=%p", p.q, p.aa, p.dyn);
    d = new(3, 4); b = d;
    $display("T|8.13|a=%0d who=%s nv=%s b.nv=%s calc=%0d", d.a, b.who(), d.nv(), b.nv(), b.calc(5));
    $display("T|8.14|shadow d.v=%0d b.v=%0d", d.v, b.v);
    dd = new; b = dd; $display("T|8.20|implicit virtual %s a=%0d", b.who(), b.a);
    if ($cast(d2, b)) $display("T|8.16|downcast ok %s", d2.who()); else $display("T|8.16|downcast fail");
    b = new(1);
    if ($cast(d2, b)) $display("T|8.16|bad downcast ok?"); else $display("T|8.16|bad downcast fail as expected");
    $display("T|8.16|cast null %0d", $cast(d2, null));
    ea = new; $display("T|8.17|extends args a=%0d", ea.a);
    n = new(1); n.next = new(2); n.next.next = new(3);
    $display("T|8.15|chain %0d %0d", n.next.next.val, n.next.val);
    o1 = new; o1.in_h.x = 5; o2 = new o1; o2.y = 9; o2.in_h.x = 7;
    $display("T|8.12|shallow o1.y=%0d o1.in.x=%0d o2.y=%0d same=%0d", o1.y, o1.in_h.x, o2.y, o1.in_h == o2.in_h);
    p2 = new p; $display("T|8.12|copy no ctor count=%0d id=%0d q=%p", Packet::count, p2.id, p2.q);
    p2.q.push_back(4); $display("T|8.12|copy deep q p=%p p2=%p", p.q, p2.q);
    begin Base bb; bb = Derived::new(1,1); $display("T|8.8|typed ctor %s", bb.who()); end
    begin Base bb; bb = new(2); bb = null; $display("T|8.24|null after %0d", bb == null); end
  end
endmodule
