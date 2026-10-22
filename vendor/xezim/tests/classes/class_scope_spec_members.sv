// Group B: member kinds through a type parameter bound to a specialization
typedef enum {RED, GREEN, BLUE} color_e;
class item; string name; function new(string n); name = n; endfunction endclass
class holder #(type T = int, string Tname = "<unknown>", int P = 4);
  localparam int LP = P * 2;
  typedef T elem_t;
  static int count = 11;
  static item inst;
  static function string type_name(); return Tname; endfunction
  static function int num(); return P + 100; endfunction
  static function color_e col(); return BLUE; endfunction
  static function item get(); if (inst == null) inst = new(Tname); return inst; endfunction
  static task bump(); count++; endtask
endclass
class via #(type R = int);
  typedef R::elem_t et_t;
  function void run();
    item it;
    et_t e;
    string s;
    $display("T|B01 bare string:%s", R::type_name);
    $display("T|B02 paren string:%s", R::type_name());
    $display("T|B03 bare int:%0d", R::num);
    $display("T|B04 bare enum:%s", R::col.name());
    $display("T|B05 bare enum cmp:%0d", R::col == BLUE);
    it = R::get;
    $display("T|B06 bare handle:%s", it.name);
    $display("T|B07 chained get().name:%s", R::get().name);
    $display("T|B08 static prop:%0d", R::count);
    $display("T|B09 bare+1:%0d", R::num + 1);
    s = {R::type_name, "!"};
    $display("T|B10 concat:%s", s);
    $display("T|B11 compare:%0d", R::type_name == "mk");
    $display("T|B12 sformatf arg:%s", $sformatf("<%s>", R::type_name));
    $display("T|B13 user fn arg:%s", wrap(R::type_name));
    R::bump;
    $display("T|B14 bare task stmt:%0d", R::count);
    R::bump();
    $display("T|B15 paren task stmt:%0d", R::count);
    $display("T|B16 param P:%0d", R::P);
    $display("T|B17 localparam LP:%0d", R::LP);
    e = 5;
    $display("T|B18 typedef elem_t bits:%0d", $bits(e));
    $display("T|B20 cond bare:%s", (R::num > 0) ? R::type_name : "no");
  endfunction
  function string wrap(string x); return {"[", x, "]"}; endfunction
endclass
module top;
  initial begin
    via #(holder #(byte, "mk", 6)) v = new();
    v.run();
  end
endmodule
