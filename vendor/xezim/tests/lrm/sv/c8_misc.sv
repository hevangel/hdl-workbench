// top: c8_misc
class Acc;
  local int secret = 5; protected int prot = 6; int pub = 7;
  const int ci = 3;               // global const, set at decl
  const int inst_c;               // instance const, set in new
  static const int sc = 11;
  function new(int c); inst_c = c; endfunction
  function int peek(Acc other); return other.secret; endfunction // same class can see local of other
endclass
class AccD extends Acc;
  function new(); super.new(4); endfunction
  function int gp(); return prot + pub; endfunction
endclass
class Cnt; static int n; int id; function new(); id = n++; endfunction endclass
class Base; virtual function void hi(); $display("T|8.20|base hi"); endfunction endclass
class Fin extends Base;
  virtual function void hi(); $display("T|8.20|fin hi"); super.hi(); endfunction
endclass
class Mid extends Fin; endclass
class Leaf extends Mid; function void hi(); $display("T|8.20|leaf hi"); super.hi(); endfunction endclass
class WithTask;
  int v;
  task automatic run(int n); repeat(n) begin #2 v++; end endtask
  static task sdelay(); #3; endtask
endclass
class Rec; int depth; function int f(int n); return n <= 1 ? 1 : n * f(n-1); endfunction endclass
class StaticInit; static int a = 4; static int b = a * 2; endclass
class Ev; event e; int hits; task waiter(); @e hits++; endtask endclass
module c8_misc;
  Acc a1, a2; AccD ad; Cnt c[3]; Base b; Leaf lf; WithTask wt; Rec r; Ev ev;
  initial begin
    a1 = new(1); a2 = new(2); ad = new;
    $display("T|8.18|peek local %0d gp=%0d pub=%0d", a1.peek(a2), ad.gp(), a1.pub);
    $display("T|8.19|const ci=%0d inst=%0d,%0d,%0d sc=%0d", a1.ci, a1.inst_c, a2.inst_c, ad.inst_c, Acc::sc);
    foreach (c[i]) c[i] = new; $display("T|8.9|ids %0d %0d %0d n=%0d", c[0].id, c[1].id, c[2].id, Cnt::n);
    lf = new; b = lf; b.hi();
    wt = new; fork wt.run(3); WithTask::sdelay(); join $display("T|8.10|task in class v=%0d t=%0t", wt.v, $time);
    r = new; $display("T|8.6|recursion %0d", r.f(5));
    $display("T|8.9|static init b=%0d", StaticInit::b);
    ev = new; fork ev.waiter(); #1 -> ev.e; join $display("T|8.5|event prop hits=%0d", ev.hits);
  end
endmodule
