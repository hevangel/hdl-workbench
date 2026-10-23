interface pif; int data; endinterface
class I;
  virtual pif v;
  virtual pif vq[$];
  virtual pif vd[];
  virtual pif vf[2];
  function int rd(int i); return vq[i].data; endfunction
endclass
class S;
  static virtual pif pa[2];
  static virtual pif pq[$];
  static function int rda(int i); return pa[i].data; endfunction
  static function virtual pif geta(int i); return pa[i]; endfunction
endclass
class G #(type T = int);
  static T one;
  static function void put(T v); one = v; endfunction
endclass
module tb;
  pif p1(); pif p2(); pif p3();
  initial begin
    virtual pif p;
    I o = new;
    S s = new;
    p1.data = 1; p2.data = 2; p3.data = 3;
    p = p3; o.vq.push_back(p1); o.vq.push_back(p2);
    p = o.vq[0]; $display("T|q0|%0d", p.data);
    p = o.vq[1]; $display("T|q1|%0d", p.data);
    o.vq.push_front(p3); p = p1; p = o.vq[0]; $display("T|qf|%0d %0d", p.data, o.vq[0].data);
    $display("T|qd|%0d %0d", o.vq[1].data, o.rd(2));
    p = p1; o.vd = new[2]; o.vd[1] = p2; p = o.vd[1]; $display("T|d1|%0d", p.data);
    p = p3; o.vf[0] = p2; p = o.vf[0]; $display("T|f0|%0d %0d", p.data, o.vf[0].data);
    p = p3; S::pa[1] = p1; S::pa[0] = p2; p = S::pa[1]; $display("T|s1|%0d", p.data);
    p = S::pa[0]; $display("T|s0|%0d %0d %0d", p.data, S::pa[1].data, S::rda(0));
    p = p3; p = s.pa[1]; $display("T|sh|%0d", p.data);
    p = p3; p = S::geta(0); $display("T|sg|%0d", p.data);
    p = p1; S::pq.push_back(p2); p = S::pq[0]; $display("T|sq|%0d %0d", p.data, S::pq[0].data);
    G#(virtual pif)::put(p2);
    $display("T|g1|%0d", G#(virtual pif)::one.data);
    G#(virtual pif)::put(p3);
    $display("T|g2|%0d", G#(virtual pif)::one.data);
    $finish;
  end
endmodule
