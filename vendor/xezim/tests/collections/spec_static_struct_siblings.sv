class P #(type T = int);
  typedef struct { string s; T v; } e_t;
  static e_t q[$];
  static e_t fa[2];
  static e_t one;
  static int cnt;
  static function void put(string s, T v);
    e_t e; e.s = s; e.v = v; q.push_back(e);
  endfunction
  static function string show(); return $sformatf("%p", q); endfunction
  static function string show1(); return $sformatf("%p", one); endfunction
endclass
typedef P#(int) pi_t;
typedef P#(byte) pb_t;
module tb;
  initial begin
    P#(int)::put("a", 7);
    $display("T|01 show=%s", P#(int)::show());
    P#(int)::fa[1].v = 12;
    P#(int)::fa[0].s = "z";
    $display("T|02 fa v=%0d s=%s", P#(int)::fa[1].v, P#(int)::fa[0].s);
    P#(int)::one.v = 13;
    P#(int)::one.s = "o";
    $display("T|03 one v=%0d s=%s", P#(int)::one.v, P#(int)::one.s);
    $display("T|03b %s", P#(int)::show1());
    pi_t::put("t", 16);
    $display("T|04 alias v=%0d s=%s n=%0d", pi_t::q[1].v, pi_t::q[1].s, pi_t::q.size());
    pi_t::cnt = 4;
    $display("T|05 cnt=%0d %0d", P#(int)::cnt, pi_t::cnt);
    pb_t::put("b", 3);
    $display("T|06 b=%0d n=%0d", P#(byte)::q[0].v, P#(byte)::q.size());
    pi_t::one.v = 21;
    $display("T|07 one=%0d", P#(int)::one.v);
  end
endmodule
