package pk;
  typedef int q_t [$];
  int a [$] = {1, 2};
  int b [$] = {3};
  int c [$] = {a, b, 9};
  q_t d = {a, b};
  int h [$] = '{5, 6};
endpackage
module tb;
  int ma [$] = {1, 2};
  int mb [$] = {3};
  int mc [$] = {ma, mb, 9};
  pk::q_t md = {ma, mb};
  initial begin
    $display("T|01 c=%p d=%p h=%p", pk::c, pk::d, pk::h);
    $display("T|03 mc=%p md=%p", mc, md);
  end
endmodule
