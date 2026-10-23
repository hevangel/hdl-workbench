// DPI import forms (IEEE 1800-2017 sec. 35.5): a `pure` import driving a
// continuous assignment and an always_comb block, re-evaluated as its input
// changes; two SV names linked to one C function (the `c_name =` form); a C-owned object carried as a chandle in a class; and one call
// writing output arguments of several types. Lines tagged `T|` are checked
// by the cargo test.
class counter;
  chandle h;
  function new(int step);
    h = c_counter_new(step);
  endfunction
  function int bump();
    return c_counter_bump(h);
  endfunction
  function void release_it();
    c_counter_free(h);
    h = null;
  endfunction
endclass

import "DPI-C" pure function int c_parity(input int v);
import "DPI-C" function int c_pure_calls();
import "DPI-C" c_sum = function int add_ints(input int a, input int b);
import "DPI-C" c_sum = function int also_add(input int a, input int b);
import "DPI-C" function chandle c_counter_new(input int step);
import "DPI-C" function int c_counter_bump(input chandle h);
import "DPI-C" function void c_counter_free(input chandle h);
import "DPI-C" function void c_outputs(input int x, output real half, output string label,
                                       output bit [7:0] byte_out, output logic [3:0] nib,
                                       output bit odd);

module top;
  int v = 0;
  wire [31:0] par_w = c_parity(v);
  int par_c;
  always_comb par_c = c_parity(v + 1);

  initial begin
    counter a, b;
    real half; string label; bit [7:0] by; logic [3:0] nib; bit odd;

    #1 $display("T|pure v=%0d wire=%0d comb=%0d", v, par_w, par_c);
    v = 7;
    #1 $display("T|pure v=%0d wire=%0d comb=%0d", v, par_w, par_c);
    v = 6;
    #1 $display("T|pure v=%0d wire=%0d comb=%0d", v, par_w, par_c);
    $display("T|pure called=%0d", c_pure_calls() > 0);

    $display("T|alias %0d %0d", add_ints(2, 3), also_add(40, 2));

    a = new(1);
    b = new(10);
    void'(a.bump()); void'(b.bump());
    $display("T|chandle a=%0d b=%0d", a.bump(), b.bump());
    a.release_it(); b.release_it();
    $display("T|released %0d", a.h == null && b.h == null);

    c_outputs(37, half, label, by, nib, odd);
    $display("T|outputs half=%0.1f label=%s byte=%0d nib=%b odd=%0d", half, label, by, nib, odd);
    c_outputs(21, half, label, by, nib, odd);   // x[4] set: nib[0] reads X
    $display("T|outputs half=%0.1f label=%s byte=%0d nib=%b odd=%0d", half, label, by, nib, odd);
    $display("T|done");
  end
endmodule
