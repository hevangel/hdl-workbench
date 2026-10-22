interface my_if #(type T = int, int N = 4);
  T data;
  logic [N-1:0] w;
endinterface
interface plain_if; int data; endinterface
class db #(type T = int);
  static T vals[$];
  static T one;
  static int cnt;
  static function void set(T v); vals.push_back(v); one = v; cnt++; endfunction
  static function int size(); return vals.size(); endfunction
endclass
typedef virtual my_if#(int) vif_int_t;
typedef int myint_t;
typedef int intq_t[$];
typedef int int2_t[2];
class mon #(type T = int, int N = 4);
  typedef virtual my_if#(T) vif_t;
  typedef T tq_t[$];
  typedef T t2_t[2];
  virtual my_if#(T) vif;
  function void check(string tag);
    $display("T|%s_size_vifT|%0d", tag, db#(virtual my_if#(T))::size());
    $display("T|%s_size_vif_t|%0d", tag, db#(vif_t)::size());
    $display("T|%s_size_vif_int|%0d", tag, db#(virtual my_if#(int))::size());
    $display("T|%s_size_T|%0d", tag, db#(T)::size());
    $display("T|%s_cnt_vifT|%0d", tag, db#(virtual my_if#(T))::cnt);
    $display("T|%s_size_vifTN|%0d", tag, db#(virtual my_if#(T, N))::size());
    $display("T|%s_size_typedef|%0d", tag, db#(vif_int_t)::size());
    db#(virtual my_if#(T))::set(vif);
  endfunction
endclass
class child #(type U = int) extends mon#(U);
  function void c2(); check("child"); endfunction
endclass
class wrap #(type T = int);
  static function int sz(); return db#(virtual my_if#(T))::size(); endfunction
  static function int szm(); return db#(myint_t)::size(); endfunction
endclass
module tb;
  my_if#(int) vi();
  my_if#(int, 8) vi8();
  my_if#(byte) vb();
  initial begin
    mon#(int) m; mon#(int, 8) m8; mon#(byte) mb; child#(int) ch;
    int q[$]; int a[2];
    vi.data = 42; vb.data = 7; vi8.data = 9;
    db#(virtual my_if#(int))::set(vi);
    db#(virtual my_if#(int, 8))::set(vi8);
    db#(virtual my_if#(byte))::set(vb);
    db#(int)::set(5);
    db#(intq_t)::set(q);
    db#(int2_t)::set(a);
    m = new; m8 = new; mb = new; ch = new;
    m.check("m");
    mb.check("mb");
    m8.check("m8");
    ch.c2();
    $display("T|wrap|%0d", wrap#(int)::sz());
    $display("T|wrapm|%0d", wrap#(int)::szm());
    $display("T|mod_after|%0d", db#(virtual my_if#(int))::size());
    $display("T|mod_after_typedef|%0d", db#(vif_int_t)::size());
    $finish;
  end
endmodule
