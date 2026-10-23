// top: c15
module c15;
  semaphore sm;
  mailbox mb;
  mailbox #(int) mbi;
  mailbox #(string) mbs;
  event e1, e2, e3;
  string lg[$];
  initial begin
    // 15.3 semaphore
    sm = new(2);
    sm.get(1);
    $display("T|15.3a|try=%0d", sm.try_get(2));
    $display("T|15.3b|try=%0d", sm.try_get(1));
    fork
      begin sm.get(2); lg.push_back($sformatf("got2@%0t", $time)); end
      begin #5 sm.put(2); lg.push_back($sformatf("put2@%0t", $time)); end
    join
    $display("T|15.3c|%p", lg);
    sm.put(5); $display("T|15.3d|try=%0d", sm.try_get(5));
    // FIFO ordering of waiters
    lg = {};
    fork
      begin sm.get(3); lg.push_back("w3"); end
      begin #1 sm.get(1); lg.push_back("w1"); end
      begin #2 sm.put(1); #1 sm.put(3); end
    join
    $display("T|15.3e|%p", lg);
    // 15.4 mailbox
    mb = new();
    mb.put(1); mb.put("str"); $display("T|15.4a|num=%0d", mb.num());
    begin int v; string s; mb.get(v); mb.get(s); $display("T|15.4b|%0d %s", v, s); end
    mbi = new(2);
    mbi.put(10); mbi.put(20);
    $display("T|15.4c|tryput=%0d num=%0d", mbi.try_put(30), mbi.num());
    begin int v; $display("T|15.4d|peek=%0d", mbi.try_peek(v)); $display("T|15.4e|v=%0d num=%0d", v, mbi.num()); end
    begin int v; mbi.get(v); $display("T|15.4f|%0d", v); mbi.get(v); $display("T|15.4g|%0d", v);
      $display("T|15.4h|tryget=%0d", mbi.try_get(v)); end
    lg = {};
    fork
      begin int v; mbi.get(v); lg.push_back($sformatf("g%0d@%0t", v, $time)); end
      begin #3 mbi.put(77); end
    join
    $display("T|15.4i|%p", lg);
    // blocking put on full
    lg = {};
    fork
      begin mbi.put(1); mbi.put(2); mbi.put(3); lg.push_back($sformatf("put3@%0t", $time)); end
      begin int v; #4 mbi.get(v); lg.push_back($sformatf("got%0d", v)); end
    join
    $display("T|15.4j|%p", lg);
    begin int v; while (mbi.try_get(v) > 0); end
    // type mismatch try_get on untyped mailbox: returns negative
    mb.put("hello");
    begin int v; $display("T|15.4k|%0d", mb.try_get(v)); end
    begin string s; void'(mb.try_get(s)); end
    mbs = new; mbs.put("x"); begin string s; mbs.peek(s); $display("T|15.4l|%s %0d", s, mbs.num()); end
    // 15.5 named events: triggered, persistence in time step
    fork
      begin -> e1; end
      begin wait (e1.triggered); $display("T|15.5.3a|triggered seen t=%0t", $time); end
    join
    #1 $display("T|15.5.3b|triggered next step=%0d", e1.triggered);
    // @ misses an event triggered earlier in same timestep (race); wait(triggered) does not
    // 15.5.5 event merging
    e3 = e2;
    fork
      begin @e3 $display("T|15.5.5a|merged e3 woke via e2 t=%0t", $time); end
      begin #1 -> e2; end
    join
    e3 = null;
    fork
      begin fork @e3 $display("T|15.5.5b|null event fired?"); join_none #3 disable fork; $display("T|15.5.5b|null never fires"); end
      begin #1 -> e2; end
    join
    // ->> nonblocking trigger
    fork
      begin @e1 $display("T|15.5.1|nb trigger seen t=%0t", $time); end
      begin #1 ->> e1; end
    join
  end
endmodule
