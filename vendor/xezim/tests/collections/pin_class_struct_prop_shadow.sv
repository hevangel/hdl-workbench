typedef struct { int q[$]; int k; } s_t;
class C;
  s_t s;
  function void fill();
    s.q.push_back(1); s.k = 5;
  endfunction
endclass
module tb;
  s_t s;
  initial begin
    C c = new();
    c.fill();
    $display("T|01 c=%p m=%p", c.s.q, s.q);
  end
endmodule
