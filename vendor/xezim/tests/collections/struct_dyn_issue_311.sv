module aa_elem_queue_member_tb;

  typedef struct { int x; int ql[$]; } elem_t;

  elem_t aa[string];
  elem_t s;                     // control: same ops on a plain struct
  int    qc[$];
  int    errors = 0;
  int    n;

  task automatic check(string what, int got, int exp);
    if (got !== exp) begin
      errors++;
      $display("[ERROR] %s: got %0d, expected %0d", what, got, exp);
    end
  endtask

  initial begin
    // Controls on a plain struct: all of these work today.
    s.ql.push_back(1);
    s.ql.push_back(2);
    check("plain struct push_back",     s.ql.size(), 2);
    s.ql = {3, 4, 5};
    check("plain struct member assign", s.ql.size(), 3);

    // 1) push_back through an AA element subscript.
    aa["b"].ql.push_back(1);
    aa["b"].ql.push_back(2);
    aa["b"].ql.push_back(3);
    check("AA elem push_back size",    aa["b"].ql.size(), 3);
    n = 0; foreach (aa["b"].ql[i]) n++;
    check("AA elem push_back foreach", n,                3);

    // 2) Direct member assignment through an AA element subscript.
    aa["a"].ql = {7, 8};
    check("AA elem member assign size", aa["a"].ql.size(), 2);
    check("AA elem member assign [1]",  aa["a"].ql[1],     8);

    // 3) Queue concatenation with AA-element member operands.
    qc = {aa["a"].ql, aa["b"].ql};
    check("concat of AA elem queues size", qc.size(), 5);
    check("concat contents sum",
          qc[0] + qc[1] + qc[2] + qc[3] + qc[4], 7 + 8 + 1 + 2 + 3);

    if (errors == 0) $display("TEST_PASS");
    else             $display("TEST_FAIL with %0d errors", errors);
    $finish;
  end
endmodule


