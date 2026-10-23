
// ============================================================================
// A static virtual-interface member of a PARAMETERIZED class never resolves
// when reached through the specialization C#(T)::svif.
//
// SYMPTOM: `a_pm_c#(int)::svif = ci;` recorded its binding under exactly the
// key the read consults, yet `a_pm_c#(int)::svif.count` came back 0, while the
// same class bound through a static method read back fine through the same
// qualified spelling.  The write and the read agreed on the key; the read could
// not reach the cell.  A static property of a specialization has ONE cell, and
// the class-scope read (handle 0) resolved only through the per-object table,
// which cannot hold a static cell; the entry gate of the chain rewriter also
// never consulted the static binding map, so a design whose only bound vifs are
// class statics was rejected before its chain could be walked at all.
//
// Staged checks:
//   A PRIMARY : qualified write, then qualified member read
//   B PRIMARY : two specializations of one class are two distinct static cells
//               (LRM 8.26 — statics are per-specialization), so the fix may not
//               collapse the spec key
//   C control : parameterized class bound through a static method, qualified read
//   D control : non-parameterized class, plain spelling both ways
//
// Each stage owns its class so no binding can leak from one stage to another.
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

// A: qualified write, qualified read
class a_pm_c #(type T = int);
  static virtual counter_if svif;
endclass

// B: one class, two specializations, two cells
class b_pm_c #(type T = int);
  static virtual counter_if svif;
endclass

// C: bound through a static method of the class
class c_pm_c #(type T = int);
  static virtual counter_if svif;
  static function void bind_it(virtual counter_if h); svif = h; endfunction
endclass

// D: control, no parameters
class d_ctl_c;
  static virtual counter_if svif;
endclass

module top;
  counter_if ci();
  counter_if cj();

  initial begin
    `SVTEST_INIT
    cj.count = 7;

    // A (PRIMARY): the qualified write must land in the specialization cell.
    a_pm_c#(int)::svif = ci;
    `SVTEST_CHECK(a_pm_c#(int)::svif.count == 42, "A member read after qualified write")

    // B (PRIMARY): each specialization owns its own static cell.
    b_pm_c#(int)::svif = ci;
    b_pm_c#(bit)::svif = cj;
    `SVTEST_CHECK(b_pm_c#(int)::svif.count == 42, "B int specialization keeps its binding")
    `SVTEST_CHECK(b_pm_c#(bit)::svif.count == 7, "B bit specialization keeps its binding")

    // C (control): method write, qualified read.
    c_pm_c#(int)::bind_it(ci);
    `SVTEST_CHECK(c_pm_c#(int)::svif.count == 42, "C qualified read after method write")

    // D (control): non-parameterized class, plain spelling.
    d_ctl_c::svif = ci;
    `SVTEST_CHECK(d_ctl_c::svif.count == 42, "D plain member read")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule