// top: c25x
interface pif #(parameter type T = logic [3:0], parameter int D = 2) ();
  T data [D];
  T single;
  function automatic T get(int i); return data[i]; endfunction
endinterface
interface nested_if; pif #(.T(byte)) inner(); logic flag; endinterface
module takes_pif(pif p); initial #2 $display("T|25.3.4|%0d %0d", $bits(p.single), p.get(1)); endmodule
class holder #(type VT = virtual pif);
  VT v;
  function new(VT x); v = x; endfunction
  function int peek(); return v.data[0]; endfunction
endclass
module c25x;
  pif #(.T(shortint), .D(3)) p1();
  pif p2();
  nested_if ni();
  takes_pif tp(p1);
  // generate array of interfaces with virtual interface array
  pif gen_ifs [3] ();
  virtual pif #(.T(shortint), .D(3)) vp;
  initial begin
    p1.data[0] = -5; p1.data[1] = 300; p1.single = 7;
    p2.single = 4'hf;
    ni.inner.single = 8'h7f; ni.flag = 1;
    vp = p1;
    #1;
    $display("T|25.10b|%0d %0d %0d", vp.data[0], vp.get(1), $bits(p2.single));
    $display("T|25.3.5|%0d %b", ni.inner.single, ni.flag);
    begin holder #(virtual pif #(.T(shortint), .D(3))) h; h = new(p1); $display("T|25.9g|%0d", h.peek()); end
    gen_ifs[0].single = 0; gen_ifs[1].single = 1; gen_ifs[2].single = 2;
    #1 $display("T|25.3.6|%0d %0d", gen_ifs[2].single, gen_ifs[0].single);
  end
endmodule
