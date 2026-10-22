// §8.25 / §9.4.2 / §15.5: task contexts and OR waiters retain event identity.
`timescale 1ns/1ps
class events #(int TAG = 0);
  static event first, second;
  static int hits;
  static time woke;
  static task leaf(); @(first); hits++; woke=$time; endtask
  static task outer(); leaf(); endtask
endclass
class receiver;
  event pulse;
endclass
module top;
  receiver h;
  int errors, or_hits, duplicate_hits, mixed_hits;
  task automatic ck(string name, bit ok);
    if(ok) $display("PASS %s",name); else begin errors++; $display("FAIL %s",name); end
  endtask
  initial begin
    h=new;
    fork
      events#(1)::outer();
      events#(2)::outer();
      begin repeat(2) begin @(events#(3)::first or events#(3)::second); or_hits++; end end
      begin @(events#(4)::first or events#(4)::first); duplicate_hits++; end
      begin @(h.pulse or events#(5)::first); mixed_hits++; end
    join_none
    #2 -> events#(1)::first;
    #1;
    ck("first specialization wake", events#(1)::hits==1 && events#(1)::woke==2);
    ck("other specialization remains parked", events#(2)::hits==0);
    -> events#(3)::second;
    -> events#(4)::first;
    -> h.pulse;
    #1;
    ck("OR second alternative", or_hits==1);
    ck("duplicate terms wake once", duplicate_hits==1);
    ck("mixed instance and static group", mixed_hits==1);
    -> events#(3)::first;
    #1;
    ck("OR rearm", or_hits==2);
    -> events#(3)::second;
    -> events#(4)::first;
    -> events#(5)::first;
    -> events#(2)::first;
    #1;
    ck("completed groups stay removed", or_hits==2 && duplicate_hits==1 && mixed_hits==1);
    ck("second specialization wake", events#(2)::hits==1 && events#(2)::woke==5);
    if(errors==0) $display("ALL PASS"); else $display("FAILED %0d",errors);
    $finish;
  end
endmodule
