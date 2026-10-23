module struct_assign_queue_member_tb;

  typedef struct { int x; int ql[$]; } st_t;

  st_t s1, s2, s3;
  int  errors = 0;
  int  n;

  task automatic check(string what, int got, int exp);
    if (got !== exp) begin
      errors++;
      $display("[ERROR] %s: got %0d, expected %0d", what, got, exp);
    end
  endtask

  initial begin
    // Control: direct member queue assignment works.
    s1.ql = {10, 20, 30};
    check("direct member assign", s1.ql.size(), 3);

    // 1) An assignment pattern must overwrite the queue member.
    s1 = '{x: 1, ql: '{7, 8}};
    check("pattern assign .x",       s1.x,         1);
    check("pattern assign .ql size", s1.ql.size(), 2);
    check("pattern assign .ql[1]",   s1.ql[1],     8);

    // 2) Stale contents must not survive a full-struct assignment.
    s2.ql = {1, 2};             // leave something behind
    s2 = '{x: 5, ql: '{9}};     // full-struct pattern assignment
    check("pattern assign replaces stale queue", s2.ql.size(), 1);
    check("stale queue contents replaced",       s2.ql[0],     9);

    // 3) Control: struct-to-struct copy must copy the queue member
    //    (source populated via direct member assignment).
    s3.x  = 5;
    s3.ql = {9};
    s2    = s3;
    check("struct copy .x",        s2.x,         5);
    check("struct copy .ql size",  s2.ql.size(), 1);
    n = 0; foreach (s2.ql[i]) n++;
    check("struct copy .ql foreach", n,          1);
    s2.ql.push_back(11);        // value semantics: source must not change
    check("copy is by value (source size)", s3.ql.size(), 1);
    check("copy is by value (copy size)",   s2.ql.size(), 2);

    if (errors == 0) $display("TEST_PASS");
    else             $display("TEST_FAIL with %0d errors", errors);
    $finish;
  end
endmodule



