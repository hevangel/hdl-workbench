// top: rbs
module rbs;
  function automatic int f1(); bit [3:0] b; b = 4'b1x0z; return b; endfunction
  function automatic int f2(); byte unsigned u; u = 200; return u; endfunction
  task automatic t1(output int o); int unsigned u; u = -1; o = (u > 0); endtask
  bit [3:0] mb;
  initial begin : named
    bit [3:0] nb;
    int o;
    nb = 4'b1x0z; mb = 4'b1x0z;
    $display("T|r1|func-local bit=%0d byte-unsigned=%0d", f1(), f2());
    t1(o); $display("T|r2|task-local int unsigned > 0 = %0d", o);
    $display("T|r3|named-block bit=%b module bit=%b", nb, mb);
    fork begin bit [3:0] fb; fb = 4'b1x0z; $display("T|r4|fork-local bit=%b", fb); end join
    for (int i = 0; i < 1; i++) begin bit [3:0] lb; lb = 4'b1x0z; $display("T|r5|loop-local bit=%b", lb); end
  end
  always @(mb) begin bit [3:0] ab; ab = 4'b01xz; $display("T|r6|always-local bit=%b", ab); end
endmodule
