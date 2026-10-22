// Group A2: statics inherited from a specialization
class holder #(type T = int, string Tname = "<unknown>");
  static function string type_name(); return Tname; endfunction
endclass
class derived extends holder #(int, "ext");
  function string f4(); return type_name(); endfunction
endclass
module top;
  initial begin
    derived dd = new();
    $display("T|A12p extends inherited paren (method):%s", dd.f4());
    $display("T|A18 derived::type_name bare:%s", derived::type_name);
    $display("T|A18p derived::type_name():%s", derived::type_name());
  end
endmodule
