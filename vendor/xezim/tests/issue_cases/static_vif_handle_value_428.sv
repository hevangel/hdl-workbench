// ============================================================================
// ISSUE — a static virtual-interface member read as a VALUE yields a
//         placeholder, not the bound interface handle
//
// SYMPTOM: member access through a static virtual interface works
// (`S::sa.count` reads the right interface), but reading the member as a
// *handle* — comparing it with `==`/`!=`, or passing it where a
// `virtual counter_if` is expected — does not see the binding.  Two static
// vifs bound to two different interfaces always compare EQUAL, and a static
// vif never compares equal to the very interface it is bound to.  That is the
// signature of a boolean placeholder (nonzero constant) being returned where
// an interface handle token is required.
//
// Staged checks:
//   A control : instance-vs-instance handles compare normally (healthy path)
//   B PRIMARY : two static vifs on DIFFERENT interfaces must be unequal
//   C PRIMARY : static vif vs an instance handle on the SAME interface, equal
//   D PRIMARY : static vif vs the interface instance itself, equal
//   E control : static vif passed as an actual argument to a virtual-formal
//               (passes today and must keep passing once the value is a handle)
//   F neighbour: after a rebind, the comparisons follow the new binding
//   G control : a local copy of a static vif vs another static vif (healthy)
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
  static virtual counter_if sa;
  static virtual counter_if sb;
  virtual counter_if ia;
  virtual counter_if ib;
endclass

module top;
  counter_if ci();
  counter_if oi();

  `SVTEST_INIT

  // E: consume a static vif where a virtual formal is expected
  function automatic int cnt_of(virtual counter_if v);
    return v.count;
  endfunction

  function automatic void compare_checks();
    holder_c h = new();
    virtual counter_if lv;
    oi.count = 7;
    holder_c::sa = ci;
    holder_c::sb = oi;
    h.ia = ci;
    h.ib = oi;
    lv = holder_c::sa;

    // both sides do resolve to the right interface for member access
    `SVTEST_CHECK(holder_c::sa.count == 42, "pre sa member read")
    `SVTEST_CHECK(holder_c::sb.count == 7, "pre sb member read")

    // A (control): the instance-handle comparison path is healthy
    `SVTEST_CHECK((h.ia != h.ib), "A instance-vs-instance unequal")
    `SVTEST_CHECK((h.ia == h.ia), "A instance self equality")

    // B (PRIMARY): two static vifs, two different interfaces
    `SVTEST_CHECK((holder_c::sa != holder_c::sb), "B static-vs-static unequal")

    // C (PRIMARY): static vif vs an instance handle on the same interface
    `SVTEST_CHECK((holder_c::sa == h.ia), "C static-vs-instance equal")

    // D (PRIMARY): static vif vs the interface instance itself
    `SVTEST_CHECK((holder_c::sa == ci), "D static-vif equals interface instance")

    // E (control): static vif as an actual argument
    `SVTEST_CHECK(cnt_of(holder_c::sb) == 7, "E static-vif as argument")

    // F (neighbour): comparisons follow a rebind
    holder_c::sa = oi;
    `SVTEST_CHECK(holder_c::sa.count == 7, "F rebind member read")
    `SVTEST_CHECK((holder_c::sa == h.ib), "F rebind static-vs-instance equal")
    `SVTEST_CHECK((holder_c::sa != h.ia), "F rebind static-vs-instance unequal")

    // G (control): a local copy of a static vif behaves as a handle
    `SVTEST_CHECK((lv != holder_c::sb), "G local-copy-vs-static unequal")
    `SVTEST_CHECK((lv == h.ia), "G local-copy-vs-instance equal")
  endfunction

  initial begin
    compare_checks();
    `SVTEST_PASSFAIL
    $finish;
  end
endmodule