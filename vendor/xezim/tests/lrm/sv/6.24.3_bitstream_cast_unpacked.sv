// top: rbc
module rbc;
  typedef byte barr_t[4];
  typedef bit [7:0] q8_t[$];
  typedef logic [3:0] n4_t[2];
  barr_t ba;
  int x32;
  q8_t qq;
  n4_t n4;
  initial begin
    ba = barr_t'(32'h01020304); $display("T|r1|%p", ba);
    x32 = int'(ba); $display("T|r2|%h", x32);
    qq = q8_t'(24'habcdef); $display("T|r3|%p", qq);
    n4 = n4_t'(8'h5a); $display("T|r4|%p", n4);
  end
endmodule
