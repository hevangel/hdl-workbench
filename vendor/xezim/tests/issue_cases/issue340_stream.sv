// A streaming concatenation whose source is a class member queue,
// q = {>>{obj.queue}}, yields a 1-element queue '{0}. The same expression
// on a local queue, or a plain q = obj.queue, copies all 3 elements.
class item_c;
  byte unsigned tx_bytes[$];
endclass

module top;
  initial begin
    item_c it = new();
    byte unsigned src[$];
    byte unsigned q_member[$];
    byte unsigned q_local[$];

    it.tx_bytes = '{8'h80, 8'h4f, 8'hfc};
    src = '{8'h80, 8'h4f, 8'hfc};

    q_member = {>>{it.tx_bytes}};
    q_local  = {>>{src}};

    $display("{>>{it.tx_bytes}} -> size=%0d %p  (expect 3 '{128, 79, 252})", q_member.size(), q_member);
    $display("{>>{src}}         -> size=%0d %p", q_local.size(), q_local);
    if (q_member == q_local) $display("PASS");
    else $display("FAIL: streaming a class member queue drops its contents");
    $finish;
  end
endmodule
