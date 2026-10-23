// top: c26pad
// §26.3 (issue #425): an unpacked-array package localparam keeps its
// per-element values outside the package scope. `lim_p::LIM[3]` const-folds
// as an unpacked dimension, and a scoped scalar localparam still sizes a
// dimension (the speculative associative-index data-type parse must not
// leave sticky diagnostics behind for either spelling).
package lim_p;
  localparam int unsigned LIM [4] = '{1, 2, 4, 8};
  localparam int unsigned N = 8;
endpackage
module c26pad;
  int unsigned mem [lim_p::LIM[3]]; // element select as the dimension (= 8)
  int unsigned ctl [lim_p::N];      // scoped scalar param as the dimension
  initial begin
    $display("T|26.3g|size(ctl)=%0d", $size(ctl));
    $display("T|26.3g|size(mem)=%0d", $size(mem));
    mem[lim_p::LIM[3] - 1] = 5;
    $display("T|26.3g|mem[7]=%0d elem=%0d", mem[7], lim_p::LIM[3]);
  end
endmodule
