interface my_if #(type T = int);
  T data;
endinterface
class db #(type T = int);
  static int cnt;
  function void ibump(); cnt++; endfunction
  static function void bump(); cnt++; endfunction
endclass
class mon #(type T = int);
  db#(virtual my_if#(T)) h;
  typedef db#(virtual my_if#(T)) db_t;
  function void run();
    db_t h2;
    h = new; h.ibump();
    h2 = new; h2.ibump();
    db_t::bump();
  endfunction
endclass
module tb;
  initial begin
    mon#(int) m = new;
    mon#(byte) mb = new;
    m.run();
    $display("T|int_after_m|%0d", db#(virtual my_if#(int))::cnt);
    $display("T|byte_after_m|%0d", db#(virtual my_if#(byte))::cnt);
    mb.run();
    $display("T|int_after_mb|%0d", db#(virtual my_if#(int))::cnt);
    $display("T|byte_after_mb|%0d", db#(virtual my_if#(byte))::cnt);
    $finish;
  end
endmodule
