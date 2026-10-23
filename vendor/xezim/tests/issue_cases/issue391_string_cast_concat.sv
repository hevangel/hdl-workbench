
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH
`define SVTEST_INIT \
int failures = 0;
`define SVTEST_CHECK(expr, msg) \
if (!(expr)) begin \
  failures++; \
  $display("FAIL @%0t : %s", $time, msg); \
end
`define SVTEST_PASSFAIL \
if (failures == 0) begin \
  $display("TEST_PASS"); \
end else begin \
  $display("TEST_FAIL count=%0d", failures); \
  $fatal(1); \
end
`endif

// Issue shape: append one-character string casts.
function automatic string copy_by_cast(string s);
  string out = "";
  byte unsigned c;
  for (int i = 0; i < s.len(); i++) begin
    c = s.getc(i);
    out = {out, string'(c)};
  end
  return out;
endfunction

// Issue reference: the same loop with substr stays correct.
function automatic string copy_by_substr(string s);
  string out = "";
  for (int i = 0; i < s.len(); i++)
    out = {out, s.substr(i, i)};
  return out;
endfunction

// Cousin E: bare cast (no concat) returned from a function.
function automatic string bare_cast(string s);
  byte unsigned c = s.getc(0);
  return string'(c);
endfunction

// Cousin F: prepend concat in a function.
function automatic string prepend_cast(string s);
  string out = "";
  byte unsigned c;
  for (int i = 0; i < s.len(); i++) begin
    c = s.getc(i);
    out = {string'(c), out};
  end
  return out;
endfunction

module top;
  initial begin
    `SVTEST_INIT
    string r;
    int bad_cast = -1;
    int bad_len = -1;
    int bad_substr = -1;
    int bad_bare = -1;
    int bad_prep = -1;
    int n;
    string gs;

    // A (control): the very first call is correct.
    r = copy_by_cast("HELLO");
    `SVTEST_CHECK(r == "HELLO", "A first cast call")

    // B/C (PRIMARY) + D/E/F: the 3000-call sweep across the cliff.
    for (n = 0; n < 3000; n++) begin
      r = copy_by_cast("HELLO");
      if (r != "HELLO" && bad_cast < 0) begin
        bad_cast = n;
        $display("DETAIL cast first wrong at call %0d, len %0d", n, r.len());
      end
      if (r.len() != 5 && bad_len < 0)
        bad_len = n;
      r = copy_by_substr("HELLO");
      if (r != "HELLO" && bad_substr < 0)
        bad_substr = n;
      r = bare_cast("HELLO");
      if ((r != "H" || r.len() != 1) && bad_bare < 0) begin
        bad_bare = n;
        $display("DETAIL bare first wrong at call %0d, len %0d", n, r.len());
      end
      r = prepend_cast("HELLO");
      if ((r != "OLLEH" || r.len() != 5) && bad_prep < 0) begin
        bad_prep = n;
        $display("DETAIL prepend first wrong at call %0d, len %0d", n, r.len());
      end
    end
    `SVTEST_CHECK(bad_cast < 0, "B all 3000 cast-concat calls correct")
    `SVTEST_CHECK(bad_len < 0, "C cast-concat len stays 5 (never 513)")
    `SVTEST_CHECK(bad_substr < 0, "D substr reference correct (3000 calls)")
    `SVTEST_CHECK(bad_bare < 0, "E bare string'(byte) correct (3000 calls)")
    `SVTEST_CHECK(bad_prep < 0, "F prepend concat correct (3000 calls)")

    // G (control): character basics.
    gs = "HELLO";
    `SVTEST_CHECK(gs.len() == 5, "G len")
    `SVTEST_CHECK(gs.substr(1, 1) == "E", "G substr")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
