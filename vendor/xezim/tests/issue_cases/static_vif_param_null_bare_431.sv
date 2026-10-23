// ============================================================================
// #431 — Static virtual interface of a PARAMETERIZED class never binds
// https://github.com/aionhw/xezim/issues/431
//
// A static property of a specialization has one cell PER SPECIALIZATION
// (§8.26), and the write side files the binding under the canonical spec key
// `C#(T)::p` that `static_prop_key_spec` builds. The class-scope READS went
// through `vif_slot`, which canonicalises via `vif_class_of_scope_name` — and
// that helper discards the signature. The cell it asked for was `C::p`, so a
// static bound as `C#(T)::p` read back null. The member-read path DOES resolve
// the spec, which is why a member read could succeed while `== null` on the
// very same static still reported unbound: the comparison classifier and the
// member read disagreed about which cell a static occupies.
//
// A bare read inside a static method of a parameterized class is the other
// half: with no `this`, the name resolves through the executing class context,
// which must carry the specialization the call was made on.
//
// Staged exactly as the issue files it — one PRIMARY pair per shape, each with
// its control, so a fix that collapses the spec key (breaking E) or that
// answers every bare read from the static (breaking the null controls) is
// caught:
//   A qualified write, qualified read
//   B qualified write, bare read from inside the class
//   C control: same shape, non-parameterized
//   D control: parameterized class bound through a static method
//   E control: two specializations of one class own two cells
//
// Reported against xezim 0.11.2 / git 011c5594 as
//   FAIL @0 : A qualified member read
//   FAIL @0 : B bare read after qualified write
//   FAIL @0 : C bare null check
//   FAIL @0 : D qualified null check after plain write
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

class a_pm_c #(type T = int);
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
  static function bit bare_is_null(); return svif == null; endfunction
endclass

class b_pm_c #(type T = int);
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
  static function bit bare_is_null(); return svif == null; endfunction
endclass

// C control: same shape, no parameters
class c_ctl_c;
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
  static function bit bare_is_null(); return svif == null; endfunction
endclass

class d_pm_c #(type T = int);
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
  static function bit bare_is_null(); return svif == null; endfunction
endclass

// E: one class, two specializations, two cells
class e_pm_c #(type T = int);
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
  static function int bare_cnt(); return svif.count; endfunction
endclass

module top;
  counter_if ci();
  counter_if oi();

  initial begin
    `SVTEST_INIT
    oi.count = 7;

    // A (PRIMARY): qualified write, qualified read.
    a_pm_c#(int)::svif = ci;
    `SVTEST_CHECK(a_pm_c#(int)::svif.count == 42, "A qualified member read")
    `SVTEST_CHECK(!(a_pm_c#(int)::svif == null), "A qualified null check")

    // B (PRIMARY): qualified write, bare read from inside the class.
    b_pm_c#(int)::svif = ci;
    `SVTEST_CHECK(b_pm_c#(int)::bare_cnt() == 42, "B bare read after qualified write")
    `SVTEST_CHECK(!b_pm_c#(int)::bare_is_null(), "B bare null check after qualified write")

    // C (control): non-parameterized class, plain spelling.
    c_ctl_c::svif = ci;
    `SVTEST_CHECK(c_ctl_c::svif.count == 42, "C plain member read")
    `SVTEST_CHECK(!c_ctl_c::bare_is_null(), "C bare null check")

    // D (control): parameterized class bound through a static method.
    d_pm_c#(int)::bind_it(ci);
    `SVTEST_CHECK(d_pm_c#(int)::svif.count == 42, "D qualified read after plain write")
    `SVTEST_CHECK(!(d_pm_c#(int)::svif == null), "D qualified null check after plain write")

    // E (control): each specialization owns its own static cell.
    e_pm_c#(int)::bind_it(ci);
    e_pm_c#(bit)::bind_it(oi);
    `SVTEST_CHECK(e_pm_c#(int)::svif.count == 42, "E int specialization")
    `SVTEST_CHECK(e_pm_c#(bit)::svif.count == 7, "E bit specialization")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule