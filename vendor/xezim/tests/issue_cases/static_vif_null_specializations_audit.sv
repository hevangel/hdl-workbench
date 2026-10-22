interface vif_if; int value; endinterface
class G #(type T=int);
  static virtual vif_if v;
  static function bit empty(); return v == null; endfunction
  static function virtual vif_if get(); return v; endfunction
endclass
module top;
  vif_if m(), n(); virtual vif_if q;
  initial begin
    m.value=17; n.value=29;
    $display("T|initial %0d %0d", G#(int)::v == null, G#(bit)::v == null);
    G#(int)::v=m; G#(bit)::v=n;
    $display("T|bound %0d %0d %0d %0d", G#(int)::v.value, G#(bit)::v.value, G#(int)::empty(), G#(bit)::empty());
    q=G#(bit)::get(); $display("T|copy %0d",q.value);
    G#(int)::v=null;
    $display("T|clear %0d %0d %0d %0d", G#(int)::v == null, G#(int)::empty(), G#(bit)::v.value, G#(bit)::empty());
    q=G#(int)::v; $display("T|clearcopy %0d",q==null);
    $finish;
  end
endmodule
