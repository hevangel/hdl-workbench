interface flag_if; logic sig = 1'b1; endinterface
module leaf; logic [3:0] v = 4'hA; endmodule
module top;
  flag_if irq();
  leaf sub();
  wire [1:0] w = 2'b10;
  logic [3:0] cnt = 4'h5;
  task go; $display("T|module task go called"); endtask
  class item;
    bit irq;
    bit [1:0] pending, enable;
    bit [3:0] sub;
    bit [1:0] w;
    int go;
    bit [3:0] cnt;
    function bit chk_bare();
      return irq != |(pending & enable);
    endfunction
    function bit chk_this();
      return this.irq != |(this.pending & this.enable);
    endfunction
    function void set_bare(bit v); irq = v; sub = 4'h3; w = 2'b01; go = 7; endfunction
    function void show();
      $display("T|show irq=%b sub=%h w=%b go=%0d eq_irq=%b eq_sub=%b eq_w=%b eq_go=%b",
               irq, sub, w, go, irq == 1'b1, sub == 4'h3, w == 2'b01, go == 7);
      $display("T|show_this irq=%b sub=%h w=%b go=%0d ne_irq=%b ne_sub=%b ne_w=%b ne_go=%b",
               this.irq, this.sub, this.w, this.go, this.irq != 1'b1, this.sub != 4'h3, this.w != 2'b01, this.go != 7);
    endfunction
  endclass
  class irq_probe;
    function bit check(item t);
      $display("T|IRQ %b PENDING %b ENABLE %b EXPECTED %b MISMATCH %b", t.irq, t.pending, t.enable,
               |(t.pending & t.enable), t.irq != |(t.pending & t.enable));
      return t.irq != |(t.pending & t.enable);
    endfunction
  endclass
  item t, u;
  irq_probe c;
  initial begin
    t = new; u = new; c = new;
    $display("T|c1 %b bare=%b this=%b", c.check(t), t.chk_bare(), t.chk_this());
    t.pending = 2'b11; t.enable = 2'b01; t.irq = 1;
    $display("T|c2 %b bare=%b this=%b", c.check(t), t.chk_bare(), t.chk_this());
    t.irq = 0;
    $display("T|c3 %b bare=%b this=%b", c.check(t), t.chk_bare(), t.chk_this());
    $display("T|eq t.irq==u.irq %b  t.irq!=u.irq %b  t.irq===u.irq %b", t.irq == u.irq, t.irq != u.irq, t.irq === u.irq);
    t.sub = 4'h3; t.w = 2'b01; t.go = 9; t.cnt = 4'h2;
    $display("T|rd sub=%h w=%b go=%0d cnt=%h", t.sub, t.w, t.go, t.cnt);
    $display("T|cmp sub==3 %b sub!=3 %b w==1 %b w!=1 %b go==9 %b cnt==2 %b cnt!=5 %b", t.sub == 4'h3, t.sub != 4'h3,
             t.w == 2'b01, t.w != 2'b01, t.go == 9, t.cnt == 4'h2, t.cnt != 4'h5);
    $display("T|cmp_hier sub.v==A %b w==2 %b cnt==5 %b irq.sig %b", sub.v == 4'hA, w == 2'b10, cnt == 4'h5, irq.sig);
    $display("T|cmp_mixed %b %b", t.sub == sub.v, t.w != w);
    u.set_bare(1);
    u.show();
    $display("T|u irq=%b sub=%h w=%b go=%0d", u.irq, u.sub, u.w, u.go);
    $display("T|u_vs_t irq %b sub %b w %b go %b", u.irq != t.irq, u.sub == t.sub, u.w == t.w, u.go != t.go);
    go();
    $display("T|still w=%b cnt=%h sub.v=%h", w, cnt, sub.v);
    if (c.check(t)) $display("T|branch mismatch"); else $display("T|branch match");
    $display("T|done");
  end
endmodule
