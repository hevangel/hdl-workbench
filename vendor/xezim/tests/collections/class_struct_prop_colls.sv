typedef struct { int q[$]; int a[string]; } s_t;
class C;
  s_t s;
  s_t sq [$];
  function void fill();
    s.q.push_back(1); s.a["x"] = 3;
    sq.push_back(s);
  endfunction
endclass
module tb;
  s_t ms; s_t mq[$];
  initial begin
    C c = new();
    c.fill();
    $display("T|01 %p %p", c.s.q, c.s.a);
    $display("T|02 %p", c.sq[0].q);
    ms.q.push_back(4); mq.push_back(ms);
    $display("T|03 %p", mq[0].q);
  end
endmodule
