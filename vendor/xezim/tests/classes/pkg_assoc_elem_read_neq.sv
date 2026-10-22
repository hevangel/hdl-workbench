// Pure-SystemVerilog self-check: reading a package-scope associative-array
// element through its qualified `pkg::arr[key]` form, then comparing it
// against a different-width value (`logic[63:0]` vs `bit[31:0]`).
//
// Regression: the index arm of `eval_expr_ctx` strips the `pkg::` qualifier
// off the array base to reuse the shared name-keyed element lookup. That
// strip CLONED the `Ident(<pkg>)` reference node -- which, once the package
// name itself had been resolved/memoized, carried `cached_resolved_name =
// <pkg>`. The cloned `<member>` node therefore inherited the package name
// and resolved EVERY element read back to the package (all-x), so ~49 of 50
// `get_reset() != pkg::uid_aa[i]` comparisons saw a 1-bit X on the RHS and
// the register reset-mirror check failed. Fix: reset the clone's
// `cached_resolved_name` so the bare member resolves fresh.
//
// When wrong, xezim reports errors~47 and TAG_FAIL; reference simulators
// report errors=0 and TAG_PASS.
package test_pkg;
  static bit [31:0] uid_aa[int unsigned];
  static int reset_val;

  class regt;
    int unsigned uid;
    function int get_inst_id();
      if (uid == 0) begin
        uid = $urandom();
        test_pkg::uid_aa[uid] = test_pkg::reset_val;
        test_pkg::reset_val++;
      end
      return uid;
    endfunction
  endclass

  class data_reg;
    int cidx;
    function logic [63:0] get_reset();
      return cidx;
    endfunction
  endclass

  class t;
    int errors;
    int idx;
    regt rids[int unsigned];
    data_reg dregs[int unsigned];

    function void build();
      int i;
      regt r;
      data_reg d;
      for (i = 0; i < 50; i++) begin
        r = new(); void'(r.get_inst_id());
        d = new(); d.cidx = i;
        rids[r.uid] = r;
        dregs[r.uid] = d;
      end
    endfunction

    function void run();
      // foreach over an associative array iterates in ascending key order.
      idx = 0;
      foreach (dregs[i]) begin
        if (dregs[i].get_reset() != test_pkg::uid_aa[i]) errors++;
        idx++;
      end
    endfunction
  endclass
endpackage

module top;
  initial begin
    static test_pkg::t o;
    o = new();
    o.build();
    o.run();
    $display("errors=%0d size=%0d", o.errors, test_pkg::uid_aa.size());
    if (o.errors == 0) $display("TAG_PASS"); else $display("TAG_FAIL");
  end
endmodule