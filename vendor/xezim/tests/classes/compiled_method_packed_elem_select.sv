// Regression: compiled class methods misread selects INTO an element of a
// member collection whose element is a multi-dimensional packed array
// (`rand bit [8:0][7:0] arr[3]`).
//
// The compiled lowering of the bare element read `arr[j]` is LoadCollElem
// (correct), but a further select `arr[j][2]` fell through to the generic
// Index tail and lowered to a 1-bit BitSelectConst of the loaded 72-bit
// element — while §7.4.1 makes `[2]` a packed ELEMENT select (the 8-bit
// slice [23:16]). The self-determined compare context
// (`foreach (arr[i]) if (arr[i][2] != i) ...`) additionally sized the
// select down to 1 bit via expr_max_width, so even a correct slice was
// truncated before the Neq.
//
// Reference semantics (verified against the reference simulator): for
// arr[j] = 72'h<j<<16> the read arr[j][2] is j; part-select neighbors stay
// untouched; const-index and `this.`-qualified forms agree.
module compiled_method_packed_elem_select;
  class c3;
    rand bit [8:0][7:0] arr[3];
    constraint c { foreach (arr[i]) arr[i][2] == i; }
    // for-loop + bare read assigned to a narrow local (the original
    // miscompiled shape: LoadCollElem + BitSelectConst).
    function bit ok();
      bit [2:0] b;
      for (int j = 0; j < 3; j++) begin
        b = arr[j][2];
        if (b != j) return 0;
      end
      return 1;
    endfunction
    // foreach + select straight into a self-determined compare context.
    function bit ok2();
      bit res; res = 1;
      foreach (arr[i]) if (arr[i][2] != i) res = 0;
      return res;
    endfunction
    // constant index, `this.`-qualified and case-eq forms.
    function bit ok3();
      if (arr[0][2] != 0) return 0;
      if (this.arr[1][2] !== 1) return 0;
      if (arr[2][2] != 2) return 0;
      return 1;
    endfunction
    // dynamic select over manually seeded elements (byte 2 of element j
    // holds j, byte 1 holds 8-j — a wrong bit-select would read 0).
    function bit ok4();
      for (int j = 0; j < 3; j++)
        if (arr[j][2] != j || arr[j][1] != (8 - j)) return 0;
      return 1;
    endfunction
  endclass
  initial begin
    c3 c = new();
    int r;
    repeat (1200) r = c.randomize();
    $display("T| rand r=%0d ok=%0d", r, c.ok() && c.ok2() && c.ok3());
    // Interpreter ground truth for the seeded shape, then compiled reads.
    c.arr[0] = 72'h000800;
    c.arr[1] = 72'h010700;
    c.arr[2] = 72'h020600;
    $display("T| seeded ref b2=%0d,%0d,%0d b1=%0d,%0d,%0d",
             c.arr[0][2], c.arr[1][2], c.arr[2][2],
             c.arr[0][1], c.arr[1][1], c.arr[2][1]);
    $display("T| seeded ok4=%0d", c.ok4());
  end
endmodule
