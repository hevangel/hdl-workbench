interface vif_if; int value; endinterface
class G #(type T=int); static virtual vif_if v; endclass
class Source;
  static function void store(virtual vif_if src); G#(int)::v=src; endfunction
  static function void give(output virtual vif_if dst, input virtual vif_if src); dst=src; endfunction
  static task give_late(output virtual vif_if dst, input virtual vif_if src); #1; dst=src; endtask
endclass
module top;
  vif_if m(), n(); virtual vif_if empty;
  G#(int) a; G#(bit) b;
  function automatic void give(output virtual vif_if v, input virtual vif_if src); v=src; endfunction
  initial begin
    m.value=17; n.value=29; a=new; b=new;
    a.v=m;
    $display("T|object-isolation %0d %0d",a.v.value,b.v==null);
    b.v=n;
    $display("T|objects %0d %0d",a.v.value,b.v.value);
    a.v=empty;
    $display("T|copyclear %0d %0d",G#(int)::v==null,G#(bit)::v.value);
    give(G#(int)::v,m);
    $display("T|output %0d %0d",G#(int)::v.value,G#(bit)::v.value);
    give(G#(int)::v,empty);
    $display("T|outputclear %0d %0d",G#(int)::v==null,G#(bit)::v.value);
    G#(int)::v=m;
    G#(int)::v=G#(byte)::v;
    $display("T|specialized-clear %0d %0d",G#(int)::v==null,G#(bit)::v.value);
    Source::give(G#(int)::v,m);
    $display("T|method-output %0d %0d",G#(int)::v.value,G#(bit)::v.value);
    Source::give(G#(int)::v,empty);
    $display("T|method-clear %0d %0d",G#(int)::v==null,G#(bit)::v.value);
    Source::give_late(G#(int)::v,m);
    $display("T|task-output %0d %0d",G#(int)::v.value,G#(bit)::v.value);
    Source::give_late(G#(int)::v,empty);
    $display("T|task-clear %0d %0d",G#(int)::v==null,G#(bit)::v.value);
    Source::store(m);
    $display("T|formal-store %0d %0d",G#(int)::v.value,G#(bit)::v.value);
    Source::store(empty);
    $display("T|formal-clear %0d %0d",G#(int)::v==null,G#(bit)::v.value);
    $finish;
  end
endmodule
