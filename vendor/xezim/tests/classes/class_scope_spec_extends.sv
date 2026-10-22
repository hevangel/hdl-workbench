// Group F: static members inherited from a specialization, called inside the derived class
class holder #(type T = int, string Tname = "<unknown>", int P = 4);
  static int count = 11;
  static function string type_name(); return Tname; endfunction
  static function int pval(); return P; endfunction
endclass
class derived extends holder #(int, "ext", 9);
  function string f1(); return type_name(); endfunction
  function string f2(); return this.type_name(); endfunction
  function string f3(); return derived::type_name(); endfunction
  static function string f4(); return type_name(); endfunction
  function int f5(); return pval(); endfunction
  function int f6(); return P; endfunction
  static function int f7(); return P; endfunction
  function string f8(); return super.type_name(); endfunction
endclass
class d2 extends derived;
  function string g1(); return type_name(); endfunction
endclass
module top;
  initial begin
    derived dd = new();
    d2 e = new();
    $display("T|F01 inherited paren in method:%s", dd.f1());
    $display("T|F02 this.f() in method:%s", dd.f2());
    $display("T|F03 derived::f() in method:%s", dd.f3());
    $display("T|F04 inherited paren in static method:%s", derived::f4());
    $display("T|F05 inherited int fn in method:%0d", dd.f5());
    $display("T|F06 inherited param P in method:%0d", dd.f6());
    $display("T|F07 inherited param P in static:%0d", derived::f7());
    $display("T|F08 super.f() in method:%s", dd.f8());
    $display("T|F09 two-level inherited:%s", e.g1());
    $display("T|F10 handle.f():%s", dd.type_name());
  end
endmodule
