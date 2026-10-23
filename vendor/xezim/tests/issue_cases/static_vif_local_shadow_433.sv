// ============================================================================
// ISSUE #433 — Uninitialised frame local fails to shadow a static vif
// https://github.com/aionhw/xezim/issues/433
//
// A static property has ONE cell, keyed to its declaring class at handle 0.
// A bare virtual-interface name inside a class method resolves against that
// cell on both sides, and neither side consulted the executing frame's locals,
// so a frame-local declaration of the same name shadowed nothing (§6.21/§8.4).
// Two defects, both needed for this case to pass:
//
//   READ  `static_vif_slot_of_bare` fell back to the class cell without
//         asking the frame, so A's never-assigned local read the bound static
//         and `svif == null` reported false.
//
//   WRITE `try_bind_virtual_iface`'s bare-name arm resolved a static method's
//         missing `this` to handle 0 — the CLASS cell — and cleared it. A
//         declaration statement synthesises its initialiser as an assignment,
//         so B's `virtual counter_if svif = null;` (and C's `svif = h;`)
//         destroyed the binding for EVERY later reader. That is why D, a
//         method with no local at all, failed too: the clobber outlived the
//         frame that caused it.
//
// The controls are what keep the fix honest: C and D must still reach the
// class static, so neither guard may widen into "a bare name never reaches a
// class static".
//
// Reported against xezim 0.11.2 / git 011c5594 as
//   FAIL @0 : A uninitialised local shadows static
//   FAIL @0 : D no local falls through to static
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

class holder_c;
  static virtual counter_if svif;

  // A (PRIMARY): bare declaration, no initializer
  static function bit uninit_local_is_null();
    virtual counter_if svif;
    return svif == null;
  endfunction

  // B (control): declaration with an initializer
  static function bit init_local_is_null();
    virtual counter_if svif = null;
    return svif == null;
  endfunction

  // C (control): local assigned an interface
  static function bit bound_local_is_null(virtual counter_if h);
    virtual counter_if svif;
    svif = h;
    return svif == null;
  endfunction

  // D (control): no local, reads the static property
  static function bit no_local_is_null();
    return svif == null;
  endfunction
endclass

module top;
  counter_if ci();

  initial begin
    `SVTEST_INIT
    holder_c::svif = ci;

    // A (PRIMARY)
    `SVTEST_CHECK(holder_c::uninit_local_is_null(), "A uninitialised local shadows static")
    // B (control)
    `SVTEST_CHECK(holder_c::init_local_is_null(), "B initialised local shadows static")
    // C (control)
    `SVTEST_CHECK(!holder_c::bound_local_is_null(ci), "C bound local reads its own handle")
    // D (control) — runs LAST on purpose: it is the surviving-static check that
    // B and C's writes must not destroy.
    `SVTEST_CHECK(!holder_c::no_local_is_null(), "D no local falls through to static")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
