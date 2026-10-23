class P #(type T = int);
  typedef struct { string s; T v; } e_t;
  static e_t q[$];
  static e_t fa[2];
  static e_t one;
  static function void put(string s, T v);
    e_t e; e.s = s; e.v = v; q.push_back(e);
  endfunction
  static function string show();
    return $sformatf("%p", q);
  endfunction
endclass
typedef P#(int) pi_t;
module tb;
  initial begin
    P#(int)::put("a", 7);
    $display("T|01 show=%s", P#(int)::show());
    P#(int)::fa[1].v = 12;
    $display("T|02 fa v=%0d", P#(int)::fa[1].v);
    P#(int)::one.v = 13;
    $display("T|03 one v=%0d", P#(int)::one.v);
    pi_t::put("t", 16);
    $display("T|04 alias v=%0d s=%s", pi_t::q[1].v, pi_t::q[1].s);
  end
endmodule
