interface vif_if; int value; endinterface
class Holder;
  static virtual vif_if v;
  static function bit formal(virtual vif_if v); return v==null; endfunction
  static function int nested(virtual vif_if h);
    int result;
    begin
      virtual vif_if v;
      result=(v==null); v=h; result+=v.value;
    end
    result+=v.value;
    return result;
  endfunction
endclass
module top;
  vif_if m(), n();
  initial begin
    m.value=17; n.value=29; Holder::v=m;
    $display("T|formal %0d %0d",Holder::formal(null),Holder::formal(n));
    $display("T|nested %0d",Holder::nested(n));
    $display("T|preserved %0d",Holder::v.value);
    $finish;
  end
endmodule
