// top: c8_deep
class Item; int v; function new(int v = 0); this.v = v; endfunction
  function Item clone(); Item c = new(v); return c; endfunction
  virtual function string show(); return $sformatf("Item(%0d)", v); endfunction
  function Item me(); return this; endfunction endclass
class Sub extends Item; string tag = "s";
  function new(int v = 1); super.new(v * 10); endfunction
  virtual function string show(); return {"Sub:", super.show()}; endfunction endclass
class Base; int ctor_seen; function new(); ctor_seen = probe(); endfunction virtual function int probe(); return 1; endfunction endclass
class Der extends Base; virtual function int probe(); return 2; endfunction endclass
class Gen #(type T = Item); T h; function new(); h = new; endfunction function T get(); return h; endfunction endclass
class Cfg; int a = 1; static Cfg inst; static function Cfg get(); if (inst == null) inst = new; return inst; endfunction endclass
class Mth; function int f(int a, int b = 5, int c = 7); return a * 100 + b * 10 + c; endfunction endclass
class Pk; typedef enum {IDLE, BUSY} st_e; st_e st = BUSY; endclass
typedef struct { int k; Item h; } rec_t;
module c8_deep;
  Item q[$]; Item aa[string]; Item arr[3]; Item it; Sub s; Der d; Gen#(Sub) g; Mth m; rec_t r; Pk pk; Item dyn[];
  initial begin
    for (int i = 0; i < 3; i++) begin it = new(i); q.push_back(it); end
    $display("T|8.4|queue of handles %0d %0d %0d size=%0d", q[0].v, q[1].v, q[2].v, q.size());
    s = new(4); q.push_back(s); $display("T|8.20|poly in queue %s %s", q[3].show(), q[0].show());
    aa["x"] = new(9); aa["y"] = s; $display("T|8.4|assoc of handles %0d %s exists=%0d", aa["x"].v, aa["y"].show(), aa.exists("z"));
    foreach (arr[i]) arr[i] = new(i + 20); $display("T|8.4|array of handles %0d", arr[2].v);
    dyn = new[2]; dyn[1] = new(5); $display("T|8.4|dyn of handles null0=%0d v1=%0d", dyn[0] == null, dyn[1].v);
    it = q[1].clone(); it.v = 99; $display("T|8.12|clone independent %0d %0d", q[1].v, it.v);
    $display("T|8.15|chained call %0d %s", q[2].me().me().v, q[3].me().show());
    d = new; $display("T|8.20|virtual call from base ctor -> %0d", d.ctor_seen);
    g = new; $display("T|8.25|class-type param %s tag=%s", g.get().show(), g.h.tag);
    Cfg::get().a = 5; $display("T|8.10|singleton %0d same=%0d", Cfg::get().a, Cfg::get() == Cfg::inst);
    m = new; $display("T|8.6|default args %0d %0d %0d", m.f(1), m.f(1, 2), m.f(.a(3), .c(9)));
    r.k = 1; r.h = new(8); $display("T|8.4|handle in struct %0d", r.h.v);
    pk = new; $display("T|8.23|class enum %s %0d", pk.st.name(), pk.st == Pk::BUSY);
    q.delete(1); $display("T|7|queue delete handles %0d %0d", q[1].v, q.size());
    $display("T|8.16|cast to int via enum %0d", $cast(pk.st, 0));
  end
endmodule
