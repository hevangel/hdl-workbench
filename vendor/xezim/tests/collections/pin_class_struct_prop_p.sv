typedef int gq_t [$];
class D;
  typedef int cq_t [$];
  typedef struct { int q[$]; int k; } ds_t;
  typedef struct { cq_t q; int k; } cs_t;
  typedef struct { gq_t q; int k; } gs_t;
  ds_t ds; cs_t cs; gs_t gs;
  function void fill();
    ds.q = {1}; cs.q = {2}; gs.q = {3};
  endfunction
endclass
module tb;
  initial begin
    D d = new();
    d.fill();
    $display("T|01 %p %p %p", d.ds, d.cs, d.gs);
    $display("T|02 %p %p %p", d.ds.q, d.cs.q, d.gs.q);
  end
endmodule
