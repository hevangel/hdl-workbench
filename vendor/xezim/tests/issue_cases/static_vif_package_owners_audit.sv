interface vif_if; int value; endinterface
package p; class Holder; static virtual vif_if v; endclass endpackage
package q; class Holder; static virtual vif_if v; endclass endpackage
module top;
  vif_if m(), n(); virtual vif_if copy;
  initial begin
    m.value=17; n.value=29; p::Holder::v=m; q::Holder::v=n;
    $display("T|owners %0d %0d",p::Holder::v.value,q::Holder::v.value);
    $display("T|identities %0d %0d",p::Holder::v==q::Holder::v,p::Holder::v==m);
    copy=p::Holder::v; $display("T|copy %0d",copy.value);
    p::Holder::v=null; $display("T|clear %0d %0d",p::Holder::v==null,q::Holder::v.value);
    $finish;
  end
endmodule
