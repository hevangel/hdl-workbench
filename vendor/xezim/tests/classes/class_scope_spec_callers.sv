// Group C: callers
package cpk;
  class holder #(type T = int, string Tname = "<unknown>", int P = 4);
    static function string type_name(); return Tname; endfunction
  endclass
  class via #(type R = int);
    static function string sget(); return R::type_name; endfunction
  endclass
  typedef holder #(int, "pkfn") h_t;
  function string pf(); return h_t::type_name; endfunction
endpackage
import cpk::*;
class base #(type R = int);
  virtual function string vget(); return R::type_name; endfunction
endclass
class child #(type R = int) extends base #(R);
  virtual function string vget(); return {"c:", R::type_name, ":", super.vget()}; endfunction
endclass
class cp #(type R = int);
  localparam int W = R::P;
  static function int w(); return W; endfunction
endclass
module top;
  typedef holder #(int, "mod") hm_t;
  initial begin
    base #(holder #(int, "virt2")) b;
    child #(holder #(int, "virt2")) c = new();
    b = c;
    $display("T|C01 virtual override:%s", b.vget());
    $display("T|C02 module initial typedef bare:%s", hm_t::type_name);
    $display("T|C03 pkg function:%s", cpk::pf());
    $display("T|C04 pkg class static:%s", cpk::via #(holder #(int, "pkcls"))::sget());
    $display("T|C05 const expr param:%0d", cp #(holder #(int, "k", 9))::w());
  end
endmodule
