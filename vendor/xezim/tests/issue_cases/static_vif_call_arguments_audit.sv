interface vif_if; int value; endinterface
class G;
  static virtual vif_if v;
  static function virtual vif_if get(); return v; endfunction
endclass
class Reader;
  static function void clear(output virtual vif_if arg); endfunction
  static task clear_late(output virtual vif_if arg); #1; endtask
  static function bit empty(virtual vif_if arg); return arg==null; endfunction
  static function int read(virtual vif_if arg); return arg.value; endfunction
endclass
module top;
  vif_if m(), n(); virtual vif_if left, right; int calls=0; bit empty_result; int read_result;
  function virtual vif_if get(); calls++; return G::v; endfunction
  function void clear_free(output virtual vif_if arg); endfunction
  function int read_free(virtual vif_if arg); return arg.value; endfunction
  initial begin
    m.value=17; n.value=29; left=m; right=n; G::v=m;
    empty_result=Reader::empty(get());
    $display("T|call-null %0d %0d",empty_result,empty_result==0);
    $display("T|method-call %0d",Reader::read(G::get()));
    read_result=read_free(get());
    $display("T|free-call %0d %0d",read_result,read_result==17);
    $display("T|conditional %0d %0d",Reader::read(1'b1 ? left : right),Reader::read(1'b0 ? left : right));
    $display("T|calls %0d",calls);
    G::v=null;
    $display("T|null-call %0d",Reader::empty(get()));
    $display("T|calls-after-null %0d",calls);
    G::v=m; clear_free(G::v);
    $display("T|free-output-null %0d",G::v==null);
    G::v=m; Reader::clear(G::v);
    $display("T|method-output-null %0d",G::v==null);
    G::v=m; Reader::clear_late(G::v);
    $display("T|task-output-null %0d",G::v==null);
    $finish;
  end
endmodule
