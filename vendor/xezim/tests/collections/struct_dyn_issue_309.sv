package pkg_init_p;
  int       scalar_i = 5;                          // control: works
  bit [7:0] vector_b = 8'hAB;                      // control: works
  int       queue_q[$] = {7, 8, 9};                // control: works

  typedef struct { int x; int y; } plain_st_t;     // no dynamic members
  plain_st_t struct_s  = '{x: 9, y: 11};           // BROKEN: stays default
  int        aa_i[string] = '{"x": 11, "y": 22};   // BROKEN: stays empty
  plain_st_t aa_s[string] = '{"x": '{x: 1, y: 2}}; // BROKEN: stays empty
endpackage

module pkg_var_init_struct_aa_tb;

  int errors = 0;
  int n;

  task automatic check(string what, int got, int exp);
    if (got !== exp) begin
      errors++;
      $display("[ERROR] %s: got %0d, expected %0d", what, got, exp);
    end
  endtask

  initial begin
    // Controls: these package initializer forms already work.
    check("pkg scalar init",        pkg_init_p::scalar_i,      5);
    check("pkg vector init",        pkg_init_p::vector_b,      8'hAB);
    check("pkg queue-literal init", pkg_init_p::queue_q.size(), 3);

    // Struct initializer in package scope is dropped.
    check("pkg struct init .x", pkg_init_p::struct_s.x, 9);
    check("pkg struct init .y", pkg_init_p::struct_s.y, 11);

    // Associative-array initializer in package scope is dropped.
    check("pkg AA init num()",        pkg_init_p::aa_i.num(),     2);
    check("pkg AA init [\"x\"]",       pkg_init_p::aa_i["x"],      11);
    n = 0; foreach (pkg_init_p::aa_i[k]) n++;
    check("pkg AA init foreach",      n,                          2);

    check("pkg AA-of-struct init num()",  pkg_init_p::aa_s.num(),     1);
    check("pkg AA-of-struct init [x].x",  pkg_init_p::aa_s["x"].x,    1);

    if (errors == 0) $display("TEST_PASS");
    else             $display("TEST_FAIL with %0d errors", errors);
    $finish;
  end
endmodule


