class stream_item;
  byte unsigned q[$];
  byte unsigned d[];
  byte unsigned f[2:0];
  function automatic logic [23:0] packed_q(); return {>>{this.q}}; endfunction
endclass
module top;
  initial begin
    stream_item o;
    stream_item z;
    byte unsigned empty[$];
    logic [23:0] p24; logic [15:0] p16; logic [39:0] p40;
    o = new(); z = new();
    o.q = '{8'h12,8'h34,8'h56};
    o.d = new[2]; o.d[0]=8'h78; o.d[1]=8'h9a;
    o.f = '{8'h12,8'h34,8'h56};
    p24 = {>>{o.q}}; $display("T|queue %h", p24);
    p24 = {<<8{o.q}}; $display("T|reverse %h", p24);
    p16 = {>>{o.q[1:2]}}; $display("T|slice %h", p16);
    p16 = {>>{o.d}}; $display("T|dynamic %h", p16);
    p24 = {>>{o.f}}; $display("T|fixed %h", p24);
    p40 = {>>{8'haa,o.q,8'hbb}}; $display("T|joined %h", p40);
    empty = {>>{z.q}};
    $display("T|empty %0d", empty.size());
    $display("T|method %h", o.packed_q());
    $finish;
  end
endmodule
