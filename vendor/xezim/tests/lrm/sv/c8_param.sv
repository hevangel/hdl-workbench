// top: c8_param
class Stack #(type T = int, int N = 4);
  T items[N]; int sp;
  static int inst_cnt;
  function new(); inst_cnt++; endfunction
  function void push(T x); items[sp++] = x; endfunction
  function T pop(); return items[--sp]; endfunction
  static function int depth(); return N; endfunction
  function int tbits(); return $bits(T); endfunction
endclass
typedef Stack#(byte, 8) BStack;
class Ext #(int W = 3) extends Stack#(logic [W-1:0], W*2);
  function int w(); return W; endfunction
endclass
class Cnt #(int I = 0);
  static int c;
  static function void bump(); c += I + 1; endfunction
endclass
class TP #(type T = int);
  typedef T elem_t;
  static function T maxv(); T m = '1; return m; endfunction
endclass
class DefOnly #(int P);  // no default (2017+ allowed)
  function int p(); return P; endfunction
endclass
class Pq #(type T = int); T q[$]; endclass
module c8_param;
  Stack s1; Stack#(string, 2) s2; BStack s3; Ext#(5) e; Stack#() s4; Stack#(int,4) s5; DefOnly#(6) dp; Pq#(real) pq;
  initial begin
    s1 = new; s1.push(10); s1.push(20); $display("T|8.25|default %0d %0d bits=%0d", s1.pop(), Stack#()::depth(), s1.tbits());
    s2 = new; s2.push("a"); s2.push("bc"); $display("T|8.25|string %s depth=%0d", s2.pop(), s2.depth());
    s3 = new; $display("T|8.25|typedef spec depth=%0d bits=%0d", BStack::depth(), s3.tbits());
    e = new; $display("T|8.25|ext W=%0d depth=%0d bits=%0d", e.w(), e.depth(), e.tbits());
    s4 = new; s5 = new;
    $display("T|8.25.1|static per spec int4=%0d str2=%0d byte8=%0d", Stack#(int,4)::inst_cnt, Stack#(string,2)::inst_cnt, BStack::inst_cnt);
    Cnt#(1)::bump(); Cnt#(1)::bump(); Cnt#(5)::bump(); Cnt#()::bump();
    $display("T|8.25.1|cnt c1=%0d c5=%0d c0=%0d", Cnt#(1)::c, Cnt#(5)::c, Cnt#(0)::c);
    $display("T|8.25|type param maxv byte=%0d bit4=%b", TP#(byte)::maxv(), TP#(bit[3:0])::maxv());
    begin TP#(shortint)::elem_t z = -3; $display("T|8.25|typedef in param %0d bits=%0d", z, $bits(z)); end
    dp = new; $display("T|8.25|no default %0d", dp.p());
    pq = new; pq.q.push_back(1.5); $display("T|8.25|real q %f", pq.q[0]);
  end
endmodule
