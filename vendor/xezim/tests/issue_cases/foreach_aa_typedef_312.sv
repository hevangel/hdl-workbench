
package foreach_aa_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];   // AA type via typedef
  aa_t   aa_via_typedef;          // BROKEN: foreach iterates garbage
  elem_t aa_direct [string];      // control: foreach correct
endpackage

module foreach_aa_typedef_tb;

  int errors = 0;
  int n;

  task automatic check(string what, int got, int exp);
    if (got !== exp) begin
      errors++;
      $display("[ERROR] %s: got %0d, expected %0d", what, got, exp);
    end
  endtask

  initial begin
    // Typedef'd AA type: num() is right, foreach is garbage.
    check("typedef'd AA num()", foreach_aa_p::aa_via_typedef.num(), 0);
    n = 0; foreach (foreach_aa_p::aa_via_typedef[k]) n++;
    check("typedef'd AA foreach count", n, 0);

    // Control: direct AA declaration is fine.
    check("direct AA num()", foreach_aa_p::aa_direct.num(), 0);
    n = 0; foreach (foreach_aa_p::aa_direct[k]) n++;
    check("direct AA foreach count", n, 0);

    if (errors == 0) $display("TEST_PASS");
    else             $display("TEST_FAIL with %0d errors", errors);
    $finish;
  end
endmodule

