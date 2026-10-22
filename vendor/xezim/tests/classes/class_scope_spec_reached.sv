// Group A: how the class is reached (bare static function, string return)
package pk;
  class pholder #(type T = int, string Tname = "<pk>");
    static function string type_name(); return Tname; endfunction
  endclass
endpackage
class holder #(type T = int, string Tname = "<unknown>");
  static function string type_name(); return Tname; endfunction
endclass
class vholder #(int N = 3);
  static function int val(); return N * 10; endfunction
endclass
class holder2 #(type T = int, string Tname = "<h2>");
  static function string type_name(); return {Tname, "/", T::type_name}; endfunction
endclass
class plain_holder;
  static function string type_name(); return "plain"; endfunction
endclass
typedef holder #(int, "td") td_t;
class via #(type R = int);
  virtual function string get_type_name(); return R::type_name; endfunction
  static function string sget(); return R::type_name; endfunction
endclass
class viav #(type R = int);
  function int get(); return R::val; endfunction
endclass
class outer;
  typedef holder #(int, "cs") reg_t;
  static function string f(); return reg_t::type_name; endfunction
endclass
class derived extends holder #(int, "ext");
  static function string f2(); return type_name; endfunction
  function string f3(); return super.type_name; endfunction
endclass
module top;
  initial begin
    via #(holder #(int, "base_test")) a = new();
    via #(holder #()) b = new();
    via #(td_t) c = new();
    via #(holder2 #(holder #(int, "in"), "out")) d = new();
    via #(plain_holder) e = new();
    via #(pk::pholder #(int, "pkq")) f = new();
    viav #(vholder #(7)) g = new();
    viav #(vholder #()) h = new();
    derived dd = new();
    $display("T|A01 param->spec virtual:%s", a.get_type_name());
    $display("T|A02 param->spec static:%s", via #(holder #(int, "st"))::sget());
    $display("T|A03 param->default spec #():%s", b.get_type_name());
    $display("T|A04 param->typedef spec:%s", c.get_type_name());
    $display("T|A05 param->nested spec:%s", d.get_type_name());
    $display("T|A06 param->plain:%s", e.get_type_name());
    $display("T|A07 param->pkg spec:%s", f.get_type_name());
    $display("T|A08 param->value spec:%0d", g.get());
    $display("T|A09 param->value default:%0d", h.get());
    $display("T|A10 class-scope typedef:%s", outer::f());
    $display("T|A14 pkg::spec direct:%s", pk::pholder #(int, "pkd")::type_name);
    $display("T|A15 direct spec:%s", holder #(int, "x")::type_name);
    $display("T|A16 direct typedef:%s", td_t::type_name);
    $display("T|A17 direct default spec:%s", holder #()::type_name);
    $display("T|A18 derived::type_name bare:%s", derived::type_name);
  end
endmodule
