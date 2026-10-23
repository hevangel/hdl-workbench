interface my_if #(type T = int, int N = 4);
  T data;
  modport mp (input data);
endinterface
class db #(type T = int);
  static int cnt;
  static function void bump(); cnt++; endfunction
  function void ibump(); cnt++; endfunction
endclass
class ext #(type T = int) extends db#(virtual my_if#(T));
  function void eb(); cnt++; endfunction
endclass
class ext2 extends db#(virtual my_if);
endclass
typedef virtual interface my_if#(int) vif_kw_t;
typedef virtual my_if#(int).mp vif_mp_t;
module tb;
  my_if#(int) vi();
  initial begin
    db#(virtual my_if#(int)) h;
    ext#(int) e;
    ext2 e2;
    db#(virtual my_if#(int))::bump();
    $display("T|noargs|%0d", db#(virtual my_if)::cnt);
    $display("T|kw|%0d", db#(virtual interface my_if#(int))::cnt);
    $display("T|named|%0d", db#(virtual my_if#(.T(int)))::cnt);
    $display("T|named_both|%0d", db#(virtual my_if#(.N(4), .T(int)))::cnt);
    $display("T|full|%0d", db#(virtual my_if#(int, 4))::cnt);
    $display("T|other_n|%0d", db#(virtual my_if#(int, 5))::cnt);
    $display("T|other_t|%0d", db#(virtual my_if#(byte))::cnt);
    $display("T|kw_typedef|%0d", db#(vif_kw_t)::cnt);
    $display("T|modport|%0d", db#(virtual my_if#(int).mp)::cnt);
    $display("T|modport_typedef|%0d", db#(vif_mp_t)::cnt);
    h = new; h.ibump();
    $display("T|after_inst|%0d", db#(virtual my_if#(int))::cnt);
    $display("T|inst_view|%0d", h.cnt);
    e = new; e.eb();
    $display("T|after_ext|%0d", db#(virtual my_if#(int))::cnt);
    e2 = new; e2.ibump();
    $display("T|after_ext2|%0d", db#(virtual my_if#(int))::cnt);
    $display("T|ext2_view|%0d", e2.cnt);
    $finish;
  end
endmodule
