interface vif_if; int value; endinterface
class G #(type T=int); static virtual vif_if v; endclass
typedef G#(int) GI;
typedef G#(bit) GB;
module top;
  vif_if m(), n(); virtual vif_if copy;
  initial begin
    m.value=17; n.value=29; GI::v=m; GB::v=n;
    $display("T|aliases %0d %0d",G#(int)::v.value,G#(bit)::v.value);
    copy=GI::v; $display("T|copy %0d",copy.value);
    GI::v=null; $display("T|clear %0d %0d",G#(int)::v==null,GB::v.value);
    $finish;
  end
endmodule
