class P #(type T = int);
  typedef struct { string s; T v; } e_t;
  static e_t fa[2];
  static e_t one;
  static function string showfa(); return $sformatf("%p", fa); endfunction
  static function string show1(); return $sformatf("%p", one); endfunction
endclass
class N;
  typedef struct { string s; int v; logic [3:0] l; } e_t;
  static e_t fa[2];
  static function string showfa(); return $sformatf("%p", fa); endfunction
endclass
class Q #(type T = int);
  typedef struct { T v; byte b; } e_t;
  static e_t fa[2];
  e_t ia[2];
  static function string showfa(); return $sformatf("%p", fa); endfunction
  function string showia(); return $sformatf("%p", ia); endfunction
endclass
module tb;
  initial begin
    Q#(int) q = new;
    Q#(logic [3:0]) ql = new;
    $display("T|00 %s | %s", P#(int)::showfa(), P#(int)::show1());
    P#(int)::fa[1].v = 12;
    $display("T|01 %s", P#(int)::showfa());
    $display("T|02 %p", P#(int)::fa);
    $display("T|03 %s", N::showfa());
    N::fa[0].v = 5;
    $display("T|04 %s %p", N::showfa(), N::fa[1]);
    $display("T|05 %s %s", Q#(int)::showfa(), q.showia());
    $display("T|06 %s %s", Q#(logic [3:0])::showfa(), ql.showia());
    Q#(byte)::fa[0].b = 3;
    $display("T|07 %s %0d", Q#(byte)::showfa(), Q#(byte)::fa[1].v);
  end
endmodule
