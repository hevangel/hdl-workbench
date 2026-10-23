// §10.7: an assignment converts the value to the type of its left-hand side,
// so an `int` class property holding 32'hffff_fff6 reads -10 and compares < 0.
class holder;
  int p;
  int unsigned q;
  function void set();
    p = 32'hffff_fff6;            // unsigned literal into a signed property
    q = -10;                      // signed value into an unsigned property
  endfunction
endclass
module top;
  int errors = 0;
  int m;
  holder h;
  task automatic ck(string label, longint got, longint want);
    if (got != want) begin errors++; $display("FAIL %s: got %0d, want %0d", label, got, want); end
    else $display("PASS %s", label);
  endtask
  initial begin
    h = new;
    h.set();
    m = 32'hffff_fff6;
    ck("module int m", m, -10);
    ck("property p", h.p, -10);
    ck("property p < 0", h.p < 0, 1);
    ck("property p + 64'sd0", h.p + 64'sd0, -10);
    ck("property q", h.q, 64'd4294967286);
    ck("property q > 0", h.q > 0, 1);
    if (errors == 0) $display("ALL PASS"); else $display("FAILED %0d", errors);
  end
endmodule
