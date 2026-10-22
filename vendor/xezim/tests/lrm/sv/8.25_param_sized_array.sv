// top: t8_25a
class SP #(int N = 4); int items[N]; endclass
class Stack #(type T = int, int N = 4);
  T items[N]; int sp;
  function void push(T x); items[sp++] = x; endfunction
  function T pop(); return items[--sp]; endfunction
endclass
module t8_25a;
  SP#(3) p; Stack s;
  initial begin
    p = new; $display("T|a|items=%p size=%0d", p.items, $size(p.items));
    s = new; s.push(10); s.push(20); $display("T|b|pop=%0d", s.pop());
  end
endmodule
