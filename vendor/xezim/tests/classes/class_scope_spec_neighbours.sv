// Group E: more neighbours (package-qualified, defaults, method callers)
package pk;
  class pholder #(type T = int, string Tname = "<pk>", int P = 4);
    static int count = 21;
    static function string type_name(); return Tname; endfunction
  endclass
  typedef pholder #(int, "pktd", 8) pk_t;
endpackage
class holder #(type T = int, string Tname = "<unknown>", int P = 4);
  static int count = 11;
  static function string type_name(); return Tname; endfunction
endclass
class dflt #(type R = holder #(int, "dflt"));
  function string g(); return R::type_name; endfunction
endclass
class user;
  function string direct(); return holder #(int, "inm")::type_name; endfunction
  function int directp(); return holder #(int, "inm", 17)::P; endfunction
  static function string pkq(); return pk::pholder #(int, "inpk")::type_name; endfunction
endclass
class reg2 #(type T = int, string Tname = "<r2>");
  static function string type_name(); return Tname; endfunction
endclass
class common #(type Tregistry = int);
  virtual function string get_type_name(); return Tregistry::type_name; endfunction
endclass
class registry #(type T = int, string Tname = "<unknown>") extends common #(registry #(T, Tname));
  static function string type_name(); return Tname; endfunction
endclass
module top;
  initial begin
    dflt d = new();
    user u = new();
    registry #(int, "uvm09") r = new();
    common #(registry #(int, "uvm09")) c;
    $display("T|E01 pkg spec P:%0d", pk::pholder #(int, "a", 9)::P);
    $display("T|E02 pkg spec static prop:%0d", pk::pholder #(int, "a", 9)::count);
    $display("T|E03 pkg typedef bare fn:%s", pk::pk_t::type_name);
    $display("T|E04 pkg typedef P:%0d", pk::pk_t::P);
    $display("T|E05 type-param default spec:%s", d.g());
    $display("T|E06 direct spec bare in method:%s", u.direct());
    $display("T|E07 direct spec P in method:%0d", u.directp());
    $display("T|E08 pkg spec bare in static method:%s", user::pkq());
    $display("T|E09 uvm09 registry via common:%s", r.get_type_name());
    c = r;
    $display("T|E10 uvm09 registry via base handle:%s", c.get_type_name());
    $display("T|E11 pkg default spec bare:%s", pk::pholder #()::type_name);
  end
endmodule
