// IEEE 1800-2017 §6.21 / §25.3: initialize static interface variables before user processes.
interface path_if;
  string literal_path = "literal";
  string instance_path = $sformatf("%m");
  function automatic string getPath();
    return instance_path;
  endfunction
endinterface

module interface_string_initializer_probe;
  path_if bus();
  initial begin
    $display("IFACE_PATH|literal=%s|path=%s|method=%s",
             bus.literal_path, bus.instance_path, bus.getPath());
    if (bus.literal_path != "literal" || bus.getPath() != "interface_string_initializer_probe.bus")
      $fatal(1, "interface string initializer did not run in its instance scope");
    $display("IFACE_PATH|done");
    $finish;
  end
endmodule
