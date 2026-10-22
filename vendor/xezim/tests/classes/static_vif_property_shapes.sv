interface pif; int data; endinterface
class S;
  static virtual pif spv;
  static function virtual pif getp(); return spv; endfunction
  static function void setp(virtual pif x); spv = x; endfunction
  function virtual pif inst_get(); return spv; endfunction
  function void inst_set(virtual pif x); spv = x; endfunction
endclass
class D extends S; endclass
class G #(type T = int);
  static T one;
  static function void put(T v); one = v; endfunction
  static function T get(); return one; endfunction
endclass
module tb;
  pif pa();
  pif pb();
  pif pz();
  initial begin
    virtual pif p;
    S s = new;
    D d = new;
    pa.data = 9; pb.data = 4; pz.data = 1;
    S::setp(pb); p = pz; p = S::spv; $display("T|static_set|%0d", p.data);
    p = pz; p = s.inst_get(); $display("T|inst_get_static|%0d", p.data);
    s.inst_set(pa); p = pz; p = S::spv; $display("T|inst_set|%0d", p.data);
    p = pz; p = s.spv; $display("T|via_handle|%0d", p.data);
    p = pz; p = D::spv; $display("T|derived|%0d", p.data);
    p = pz; p = d.getp(); $display("T|derived_fn|%0d", p.data);
    S::spv = pb; p = pz; p = S::getp(); $display("T|direct_write|%0d", p.data);
    G#(virtual pif)::put(pa);
    p = pz; p = G#(virtual pif)::get(); $display("T|gen_get|%0d", p.data);
    p = pz; p = G#(virtual pif)::one; $display("T|gen_one|%0d", p.data);
    G#(int)::put(5);
    $display("T|gen_int|%0d", G#(int)::one);
    S::spv = null;
    $display("T|null|%0d", S::spv == null);
    $finish;
  end
endmodule
