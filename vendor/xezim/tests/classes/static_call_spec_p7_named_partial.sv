module top;
  class cfg #(int AW = 32, int DW = 32);
    int tag = 7;
  endclass
  class wrap #(type T = int);
    T inner;
  endclass
  class cfgs #(string S = "x", int AW = 32);
    int tag = 9;
  endclass
  class db #(type T = int);
    static T value;
    static function void set(T v); value = v; endfunction
    static function T get(); return value; endfunction
  endclass
  class agent #(int AW = 32, int DW = 32);
    function void probe();
      if (db#(cfg#(.DW(DW),.AW(AW)))::get() == null) $display("T|p7_named_partial %0d %0d got=null", AW, DW);
      else $display("T|p7_named_partial %0d %0d got=set", AW, DW);
    endfunction
  endclass
  agent#(12,32) a;
  agent#(49,128) b;
  initial begin
    cfg#(12,32) s1; cfg#(49,128) s2;
    s1 = new; s2 = new;
    db#(cfg#(12,32))::set(s1);
    a = new; b = new;
    a.probe(); b.probe();
    db#(cfg#(49,128))::set(s2);
    a.probe(); b.probe();
    $finish;
  end
endmodule
