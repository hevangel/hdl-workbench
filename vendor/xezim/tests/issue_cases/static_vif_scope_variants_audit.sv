interface vif_if; int value; endinterface
class Holder;
  static virtual vif_if v;
  static function int local_outer(virtual vif_if first, virtual vif_if second);
    virtual vif_if v=first;
    begin
      virtual vif_if v;
      $display("T|local-null %0d",v==null);
      v=second;
      $display("T|local-inner %0d",v.value);
    end
    return v.value;
  endfunction
  static function int formal_outer(virtual vif_if v, virtual vif_if second);
    begin
      virtual vif_if v;
      $display("T|formal-null %0d",v==null);
      v=second;
      $display("T|formal-inner %0d",v.value);
    end
    return v.value;
  endfunction
  static task delayed(virtual vif_if v, virtual vif_if second);
    begin
      virtual vif_if v;
      #1; $display("T|delay-null %0d",v==null);
      v=second;
      #1; $display("T|delay-inner %0d",v.value);
    end
    $display("T|delay-outer %0d",v.value);
  endtask
  static function int loop_scope(virtual vif_if second);
    int total=0;
    for(int k=0;k<2;k++) begin
      begin
        virtual vif_if v;
        total+=(v==null);
        v=second; total+=v.value;
      end
      total+=v.value;
    end
    return total;
  endfunction
  function int instance_scope(virtual vif_if second);
    int total;
    begin
      virtual vif_if v;
      total=(v==null); v=second; total+=v.value;
    end
    return total+v.value;
  endfunction
  static function bit return_inner();
    begin virtual vif_if v; return v==null; end
  endfunction
endclass
module top;
  vif_if m(), n(); Holder holder;
  initial begin
    holder=new; m.value=17; n.value=29; Holder::v=m;
    $display("T|local-outer %0d",Holder::local_outer(m,n));
    $display("T|formal-outer %0d",Holder::formal_outer(m,n));
    Holder::delayed(m,n);
    $display("T|loop %0d",Holder::loop_scope(n));
    $display("T|instance %0d",holder.instance_scope(n));
    $display("T|return %0d %0d",Holder::return_inner(),Holder::v.value);
    $finish;
  end
endmodule
