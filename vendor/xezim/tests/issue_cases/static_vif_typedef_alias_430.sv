// ============================================================================
// ISSUE — a static virtual-interface member reached through a TYPEDEF ALIAS
//         of its class never resolves
//
// SYMPTOM: after `typedef box_c box_t;`, the qualified read `box_t::svif`
// behaves as if unbound even when the very same static cell was bound through
// the real class name.  This is the READ side: writing through the alias
// (`box_t::svif = ci`) and reading back through the real class name works, so
// the alias is accepted on assignment but not on lookup.
//
// Staged checks:
//   A PRIMARY : bind through the REAL class name (static setter), read back
//               through the typedef alias — member read and null check
//   B control : bind through the typedef alias, read back through the real
//               class name (write side already works)
//   C control : real class name both ways
//
// Each stage owns its class plus its own typedef so no binding can leak.
// ============================================================================

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

interface counter_if;
  int unsigned count = 42;
endinterface

// A: bound through the real name, read through the alias
class a_real_c;
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
endclass
typedef a_real_c a_alias_t;

// B: bound through the alias, read through the real name
class b_real_c;
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
endclass
typedef b_real_c b_alias_t;

// C: real name both ways
class c_real_c;
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
  static function bit svif_is_null(); return svif == null; endfunction
endclass
typedef c_real_c c_alias_t;

module top;
  counter_if ci();

  initial begin
    `SVTEST_INIT

    // A (PRIMARY): plain write, aliased read.
    a_real_c::bind_it(ci);
    `SVTEST_CHECK(a_alias_t::svif.count == 42, "A member read through alias")
    `SVTEST_CHECK(!(a_alias_t::svif == null), "A null check through alias")

    // B (control): aliased write, plain read.
    b_alias_t::svif = ci;
    `SVTEST_CHECK(b_real_c::bare_cnt() == 42, "B bare read after aliased write")
    `SVTEST_CHECK(b_real_c::svif.count == 42, "B plain member read after aliased write")

    // C (control): real class name both ways.
    c_real_c::bind_it(ci);
    `SVTEST_CHECK(c_real_c::svif.count == 42, "C plain member read")
    `SVTEST_CHECK(!c_real_c::svif_is_null(), "C plain null check")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule