// Self-test for $typename of parameterized classes (IEEE 1800-2017 §21.7).
// $typename must return "class <name>" with specialization "#(<args>)",
// where type args render as "class <name>" (recursive) and value args as
// their literal. Mirrors the UVM `uvm_typename` / the xezim UVM support case case.
class xyz; endclass
class bar #(type T=int); endclass
class foo #(type T=int, int W=24) extends T; endclass

module top;
  initial begin
    foo #(bar#(xyz),88) f;
    bar #(xyz) b;
    f = new;
    b = f;
    if ($typename(f) == "class foo #(class bar #(class xyz), 88)")
      $display("TAG_PASS");
    else
      $display("TAG_FAIL f=[%s]", $typename(f));
    if ($typename(b) != "class bar #(class xyz)")
      $display("TAG_FAIL b=[%s]", $typename(b));
  end
endmodule
