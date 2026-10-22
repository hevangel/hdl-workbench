// DPI audit, family E: named and default actuals of an import call (named
// ones bound nothing, omitted ones passed 0), and svGetScope in a context
// import called from a package, interface, generate-block or other
// instance's function (each named the calling module). The expected values
// are the reference simulator's, loading this same C file.
package abd_pkg;
  import "DPI-C" context function string abd_pkg_where();
  function automatic string pf(); return abd_pkg_where(); endfunction
endpackage

interface abd_if;
  import "DPI-C" context function string abd_where();
  function automatic string where(); return abd_where(); endfunction
endinterface

module abd_sub;
  import "DPI-C" context function string abd_where();
  function automatic string where(); return abd_where(); endfunction
endmodule

module dpi_audit_binding;
  import abd_pkg::*;
  import "DPI-C" function int abd_args(input int a, input int b = 7);
  import "DPI-C" context function string abd_where();
  abd_if u_if();
  abd_sub u_a();
  abd_sub u_b();
  generate if (1) begin : g
    import "DPI-C" context function string abd_where();
    function automatic string where(); return abd_where(); endfunction
  end endgenerate
  int f = 0;
  `define CK(e, m) if (!(e)) begin f++; $display("FAIL %s", m); end
  initial begin
    `CK(abd_args(.b(5), .a(6)) == 65, "named actuals")
    `CK(abd_args(1) == 17, "default actual")
    `CK(abd_args(1, 2) == 12, "positional actuals")
    `CK(abd_args(.a(4)) == 47, "named actual and a default")
    `CK(abd_where() == "dpi_audit_binding", "module scope")
    `CK(abd_pkg::abd_pkg_where() == "abd_pkg", "package import, qualified call")
    `CK(abd_pkg_where() == "abd_pkg", "package import, imported name")
    `CK(abd_pkg::pf() == "abd_pkg", "package function")
    `CK(u_if.where() == "dpi_audit_binding.u_if", "interface function")
    `CK(g.where() == "dpi_audit_binding.g", "generate-block function")
    `CK(u_a.where() == "dpi_audit_binding.u_a" && u_b.where() == "dpi_audit_binding.u_b", "instance functions")
    if (f == 0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
