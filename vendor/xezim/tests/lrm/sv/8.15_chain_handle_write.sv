// top: t8_15
class Node; int val; Node next; function new(int v); val = v; endfunction endclass
module t8_15;
  Node n;
  initial begin
    n = new(1); n.next = new(2); n.next.next = new(3);
    $display("T|a|n.val=%0d n.next.val=%0d n.next.next.val=%0d", n.val, n.next.val, n.next.next.val);
    $display("T|b|n.next.next==n %0d", n.next.next == n);
  end
endmodule
