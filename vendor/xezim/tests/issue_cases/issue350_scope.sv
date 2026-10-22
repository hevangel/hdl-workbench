// IEEE 1800-2017 §6.21: each block of each procedure has its own variables;
// a variable of a nested block is not the same-named variable of another
// procedure.
// Run: xezim block_var_storage.sv
module top;
  int errors = 0;
  int seen_a, seen_b;
  event e;

  always @(e) begin
    int c;
    c = c + 1;
    seen_a = c;
  end

  always @(e) begin
    static int n = 10;
    n++;
    seen_b = n;
  end

  task check(string what, int got, int want);
    if (got != want) begin
      errors++;
      $display("FAIL %s: got %0d, want %0d", what, got, want);
    end else $display("PASS %s", what);
  endtask

  initial begin
    for (int i = 0; i < 2; i++) begin
      int c;
      static int n = 0;
      c = 5;
      n++;
    end
    -> e; #1;
    check("always block's c is its own variable", seen_a, 1);
    check("always block's n is its own variable", seen_b, 11);
    if (errors == 0) $display("ALL PASS"); else $display("%0d FAILED", errors);
  end
endmodule
