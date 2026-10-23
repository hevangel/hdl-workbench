// ============================================================================
// ISSUE — a BARE read of a static virtual-interface member inside an INSTANCE
//         method reports null while bound
//
// SYMPTOM: with a live `this`, the bare name `svif` inside an instance method is
// resolved as an instance property and looked up in the per-object virtual
// interface table, so it misses the static slot that the assignment
// `holder_c::svif = ci` filled.  The identical bare read inside a STATIC method
// is fine, and a member read through the same instance method
// (`return svif.count;`) is fine — only the scalar / null read is wrong.
//
// Staged checks:
//   A control : bare read inside a STATIC method (healthy)
//   B control : member read inside an INSTANCE method (healthy)
//   C PRIMARY : null check inside an INSTANCE method
//   D neighbour: this-qualified null check inside an INSTANCE method
//   E neighbour: instance method of a DERIVED class reading the inherited static
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

class base_c;
  static virtual counter_if svif;

  // A (control): bare read in a static method
  static function bit st_is_null();
    return svif == null;
  endfunction
  // B (control): member read in an instance method
  function int inst_count();
    return svif.count;
  endfunction
  // C (PRIMARY): null check in an instance method
  function bit inst_is_null();
    return svif == null;
  endfunction
  // D (neighbour): this-qualified null check in an instance method
  function bit inst_this_is_null();
    return this.svif == null;
  endfunction
endclass

class derived_c extends base_c;
  // E (neighbour): instance method of the derived class, inherited static
  function bit der_is_null();
    return svif == null;
  endfunction
endclass

module top;
  counter_if ci();

  initial begin
    `SVTEST_INIT
    base_c b = new();
    derived_c d = new();
    base_c::svif = ci;

    // A (control)
    `SVTEST_CHECK(!base_c::st_is_null(), "A static-method bare read")
    // B (control)
    `SVTEST_CHECK(b.inst_count() == 42, "B instance-method member read")
    // C (PRIMARY)
    `SVTEST_CHECK(!b.inst_is_null(), "C instance-method bare null check")
    // D (neighbour)
    `SVTEST_CHECK(!b.inst_this_is_null(), "D instance-method this-qualified null check")
    // E (neighbour)
    `SVTEST_CHECK(!d.der_is_null(), "E derived instance-method bare null check")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule