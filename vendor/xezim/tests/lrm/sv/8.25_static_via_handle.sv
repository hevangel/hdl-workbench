// top: t8_25b
class St #(int N = 4); static function int depth(); return N; endfunction function int d2(); return N; endfunction endclass
module t8_25b;
  St#(2) c;
  initial begin c = new; $display("T|a|handle.static=%0d handle.nonstatic=%0d scoped=%0d", c.depth(), c.d2(), St#(2)::depth()); end
endmodule
