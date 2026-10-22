// ============================================================================
// ISSUE — an element of a NON-STATIC virtual-interface array or associative
//           array reports null while bound
//
// SYMPTOM: for an ordinary (non-static) virtual-interface class property that
// is a fixed array or an associative array, the element reads back through the
// member-access path correctly (`c.arr[0].count` gives the interface value) but
// the same element compared against null says null.  A plain scalar handle, a
// queue element and a STATIC array element all behave correctly, so this is the
// element-qualified INSTANCE vif lookup missing the `name[i]` key form — not a
// static-vif problem at all.
//
// Staged checks:
//   A control : plain scalar instance handle
//   B control : queue element
//   C control : static array element
//   D PRIMARY : fixed-array element — member read and null check
//   E PRIMARY : associative-array element — member read and null check
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

class coll_c;
  virtual counter_if iv;
  virtual counter_if arr[2];
  virtual counter_if iq[$];
  virtual counter_if iaa[string];
  static virtual counter_if sarr[2];
endclass

module top;
  counter_if ci();

  initial begin
    `SVTEST_INIT
    coll_c c = new();

    // A (control): plain scalar handle
    c.iv = ci;
    `SVTEST_CHECK(c.iv.count == 42, "A plain handle member")
    `SVTEST_CHECK(!(c.iv == null), "A plain handle null check")

    // B (control): queue element
    c.iq.push_back(ci);
    `SVTEST_CHECK(c.iq[0].count == 42, "B queue element member")
    `SVTEST_CHECK(!(c.iq[0] == null), "B queue element null check")

    // C (control): static array element
    c.sarr[0] = ci;
    `SVTEST_CHECK(c.sarr[0].count == 42, "C static array element member")
    `SVTEST_CHECK(!(c.sarr[0] == null), "C static array element null check")

    // D (PRIMARY): fixed-array element
    c.arr[0] = ci;
    `SVTEST_CHECK(c.arr[0].count == 42, "D fixed-array element member")
    `SVTEST_CHECK(!(c.arr[0] == null), "D fixed-array element null check")

    // E (PRIMARY): associative-array element
    c.iaa["a"] = ci;
    `SVTEST_CHECK(c.iaa["a"].count == 42, "E assoc element member")
    `SVTEST_CHECK(!(c.iaa["a"] == null), "E assoc element null check")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule