// IEEE 1800-2017 §25.9: a parameterized virtual interface class property.
// Standalone reproducer; no UVM or vendor VIP required.
interface bus_if #(parameter int WIDTH = 8, LANES = 1);
endinterface

class agent;
  virtual interface bus_if #(.WIDTH(16), .LANES(2)) vif;
endclass

module top;
  initial begin
    $display("PASS");
    $finish;
  end
endmodule
