// DPI scopes (IEEE 1800-2017 sec. 35.5.3, 36.6): a context import runs in
// the scope of the instance it is called through or from; scopes are found
// and named by hierarchical path; svSetScope swaps and restores the current
// scope; and C -> SV -> C recursion through a top-level export. Lines
// tagged `T|` are checked by the cargo test.
module unit ();
  import "DPI-C" context function string c_scope_name();
  import "DPI-C" context function string c_switch(input string path);
  // called from inside the instance, not through a path
  initial #1 $display("T|self %m=%s", c_scope_name());
endmodule

module top;
  import "DPI-C" context function int c_fact(input int n);
  import "DPI-C" context function string c_lookup(input string path);
  import "DPI-C" context function string c_lookup_self();
  export "DPI-C" function sv_fact;

  function int sv_fact(input int k);
    return c_fact(k);
  endfunction

  unit ua ();
  unit ub ();

  initial begin
    $display("T|names a=%s b=%s", ua.c_scope_name(), ub.c_scope_name());
    $display("T|top=%s", c_lookup_self());
    $display("T|lookup %s | %s | %s", c_lookup("top.ua"), c_lookup("top.ub"), c_lookup("top"));
    $display("T|switch %s", ua.c_switch("top.ub"));
    $display("T|fact 1=%0d 5=%0d 10=%0d", c_fact(1), c_fact(5), c_fact(10));
    #2 $display("T|done");
  end
endmodule
