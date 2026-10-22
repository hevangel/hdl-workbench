// Bare `randomize()` (no `this.`) inside a class METHOD must dispatch to the
// §18.11 solver and honor the object's constraints — exactly like the
// qualified `this.randomize()` form.
module bare_rand;
  class boxx;
    rand int cnt;
    constraint cnt_c { cnt inside {[5:10]}; }
    // bare unqualified randomize() inside a method == this.randomize()
    function int draw();
      int ok = 0;
      for (int i = 0; i < 400; i++) begin
        if (!randomize()) ok = -1;      // must never fail
        else if (cnt >= 5 && cnt <= 10) ok++;
      end
      return ok;
    endfunction
    function int draw_qualified();
      int ok = 0;
      for (int i = 0; i < 400; i++) begin
        if (!this.randomize()) ok = -1;
        else if (cnt >= 5 && cnt <= 10) ok++;
      end
      return ok;
    endfunction
  endclass

  initial begin
    automatic boxx rw = new();
    int b, q;
    b = rw.draw();
    q = rw.draw_qualified();
    $display("TAG_BARESOLVE bare=%0d qualified=%0d", b, q);
    if (b == 400 && q == 400) $display("TAG_PASS bare=%0d qualified=%0d", b, q);
    else $display("TAG_FAIL bare=%0d qualified=%0d", b, q);
  end
endmodule