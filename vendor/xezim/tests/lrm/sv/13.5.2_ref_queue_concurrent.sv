// top: rrc
module rrc;
  string lg[$];
  int cnt;
  task automatic push(int d, string nm, ref string q[$]); #d q.push_back(nm); endtask
  task automatic incr(int d, ref int c); #d c = c + 1; endtask
  initial begin
    fork
      push(3, "A", lg);
      push(1, "B", lg);
    join
    $display("T|r1|%p", lg);
    cnt = 0;
    fork
      incr(3, cnt);
      incr(1, cnt);
    join
    $display("T|r2|%0d", cnt);
    cnt = 0;
    fork
      incr(2, cnt);
      #1 $display("T|r3|during=%0d", cnt);
      #3 $display("T|r4|after=%0d", cnt);
    join
    fork
      incr(2, cnt);
      #1 cnt = 100;
    join
    $display("T|r5|%0d", cnt);
  end
endmodule
