// IEEE 1800-2017 §6.21 / §25.3: preserve per-instance scope and initialize once.
package init_path_pkg;
  string label = $sformatf("%m.label");
endpackage

interface hierarchy_path_if #(parameter ID = 0);
  string instance_path = $sformatf("%m");
  string tag = $sformatf("%m:%0d", ID);
  string literal = "literal";
  string copied = $sformatf("%s", literal);
  int constant_width = $clog2(16);
  int calls = 0;
  int initialized_once = next_value();
  function automatic int next_value();
    calls++;
    return calls;
  endfunction
  function automatic string getPath(); return instance_path; endfunction
  initial begin
    if (instance_path == "" || copied != "literal" || constant_width != 4 ||
        calls != 1 || initialized_once != 1)
      $fatal(1, "interface startup state is invalid");
  end
endinterface

interface outer_path_if;
  hierarchy_path_if #(.ID(7)) inner();
endinterface

module path_holder;
  string instance_path = $sformatf("%m");
  hierarchy_path_if #(.ID(8)) bus();
endmodule

module interface_string_hierarchy_probe;
  hierarchy_path_if first();
  hierarchy_path_if #(.ID(9)) second();
  hierarchy_path_if #(.ID(3)) arrayed[2]();
  outer_path_if outer();
  path_holder holder();
  for (genvar i = 0; i < 2; i++) begin : nodes
    hierarchy_path_if #(.ID(i + 10)) bus();
  end
  if (0) begin : inactive
    hierarchy_path_if dead();
  end
  virtual hierarchy_path_if vif;

  function automatic void check(string label, string actual, string expected);
    $display("IFACE_PATH|%s=%s", label, actual);
    if (actual != expected) $fatal(1, "wrong initializer for %s", label);
  endfunction

  initial begin
    check("first", first.instance_path, "interface_string_hierarchy_probe.first");
    check("second", second.tag, "interface_string_hierarchy_probe.second:9");
    check("array0", arrayed[0].getPath(), "interface_string_hierarchy_probe.arrayed[0]");
    check("array1", arrayed[1].getPath(), "interface_string_hierarchy_probe.arrayed[1]");
    check("nested", outer.inner.getPath(), "interface_string_hierarchy_probe.outer.inner");
    check("module", holder.instance_path, "interface_string_hierarchy_probe.holder");
    check("module_interface", holder.bus.tag, "interface_string_hierarchy_probe.holder.bus:8");
    check("generate0", nodes[0].bus.tag, "interface_string_hierarchy_probe.nodes[0].bus:10");
    check("generate1", nodes[1].bus.tag, "interface_string_hierarchy_probe.nodes[1].bus:11");
    check("package", init_path_pkg::label, "init_path_pkg.label");
    vif = first;
    check("virtual", vif.getPath(), "interface_string_hierarchy_probe.first");
    if (first.calls != 1 || second.calls != 1 || arrayed[0].calls != 1 ||
        arrayed[1].calls != 1 || outer.inner.calls != 1 || holder.bus.calls != 1 ||
        nodes[0].bus.calls != 1 || nodes[1].bus.calls != 1)
      $fatal(1, "a side-effecting initializer ran more than once");
    $display("IFACE_PATH|initializers_once");
    $display("IFACE_PATH|done");
    $finish;
  end
endmodule
