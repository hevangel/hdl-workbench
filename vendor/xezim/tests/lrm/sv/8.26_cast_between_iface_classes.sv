// top: t8_26
interface class IPut; pure virtual function void put(int x); endclass
interface class IGet; pure virtual function int get(); endclass
class Fifo implements IPut, IGet;
  int q[$];
  virtual function void put(int x); q.push_back(x); endfunction
  virtual function int get(); return q.pop_front(); endfunction
endclass
module t8_26;
  Fifo f; IPut ip; IGet ig; Fifo f2;
  initial begin
    f = new; ip = f; ip.put(6);
    $display("T|a|cast iface->iface=%0d", $cast(ig, ip));
    if (ig != null) $display("T|b|get=%0d", ig.get());
    $display("T|c|cast iface->class=%0d", $cast(f2, ip));
  end
endmodule
