interface flag_if; logic sig = 1'b1; endinterface
module sub_m; flag_if irq(); bit irq_b = 1; endmodule
module top;
  flag_if irq();
  flag_if other();
  sub_m s();
  class item;
    bit irq;
    bit [1:0] pending;
  endclass
  class holder;
    virtual flag_if vif;
    virtual flag_if irq;   // a vif property with the instance's name
    bit flag;
    function bit local_copy(item t);
      bit b;
      b = t.irq;
      return b != 1'b0;
    endfunction
    function bit bare_vs(item t);
      bit irq_l;
      irq_l = t.irq;
      return irq_l == t.irq;
    endfunction
  endclass
  class item2;
    bit irq;
    function bit copy_bare();
      bit b;
      b = irq;
      return b != 1'b1;
    endfunction
  endclass
  item t; holder h; item2 i2;
  initial begin
    t = new; h = new; i2 = new;
    $display("T|vif_null %b %b", h.vif == null, h.irq == null);
    h.vif = irq; h.irq = other;
    $display("T|vif_bound %b %b %b %b", h.vif == null, h.vif != null, h.vif == h.irq, h.vif != h.irq);
    h.irq = irq;
    $display("T|vif_same %b %b", h.vif == h.irq, h.vif != h.irq);
    $display("T|local_copy0 %b", h.local_copy(t));
    t.irq = 1;
    $display("T|local_copy1 %b bare_vs %b", h.local_copy(t), h.bare_vs(t));
    i2.irq = 0;
    $display("T|copy_bare0 %b", i2.copy_bare());
    i2.irq = 1;
    $display("T|copy_bare1 %b", i2.copy_bare());
    $display("T|mixed %b %b", t.irq != i2.irq, t.irq == i2.irq);
    t.irq = 0;
    $display("T|mixed2 %b %b", t.irq != i2.irq, t.irq == i2.irq);
    $display("T|hier %b %b", s.irq_b == 1'b1, s.irq.sig != 1'b0);
  end
endmodule
