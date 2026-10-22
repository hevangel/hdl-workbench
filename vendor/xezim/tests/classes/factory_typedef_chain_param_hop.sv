// A typedef chain whose middle hop names a parameterized specialization
// (`typedef pm#(8) p8; typedef p8 p8a;`) keeps that specialization's
// parameters: p8a::type_id must be pm#(8)'s registry (IEEE 1800-2023 §6.18,
// §8.25), not the default pm#(4).
package pm_pkg;
  class registry #(type T = int);
    static int calls;
    static function T create(string name = "");
      T obj; calls++; obj = new(name); return obj;
    endfunction
    static function int count(); return calls; endfunction
  endclass
  class pm #(int W = 4);
    typedef registry#(pm#(W)) type_id;
    int w;
    function new(string name = ""); w = W; endfunction
  endclass
  typedef pm#(8) p8;
  typedef p8 p8a;
  typedef p8a p8b;
endpackage
module top;
  import pm_pkg::*;
  pm#(8) a, b;
  initial begin
    a = p8a::type_id::create("a");
    b = p8b::type_id::create("b");
    // Only the specialization is checked here. Whether `create` through a
    // parameterized alias runs the registry body is a separate open bug.
    $display("T|a.w=%0d b.w=%0d", a.w, b.w);
    $finish;
  end
endmodule
