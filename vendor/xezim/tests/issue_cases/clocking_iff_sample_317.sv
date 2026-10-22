// @(cb iff (cb.sig === v)) evaluates the iff against the PREVIOUS
// edge's sampled value. IEEE 1800-2017 14.13: clockvars are updated when
// the clocking event occurs, before processes waiting on it resume, so the
// wait should complete on the first edge at which cb.cs_n samples 0.
interface bus_if (input logic clk);
  logic        cs_n;
  logic [15:0] data;
  clocking cb @(posedge clk);
    default input #1step;
    input cs_n;
    input data;
  endclocking
endinterface

class monitor_c;
  virtual bus_if vif;
  task run();
    @(vif.cb iff (vif.cb.cs_n === 1'b0));
    $display("[%0t] woke: cb.cs_n=%b cb.data=0x%04h", $time, vif.cb.cs_n, vif.cb.data);
    if (vif.cb.data === 16'hf000 && $time == 25) $display("PASS");
    else $display("FAIL: expected to wake at 25 with cb.data=0xf000 (first beat)");
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;  // posedges at 5, 15, 25, 35, ...

  bus_if bus (clk);

  initial begin
    bus.cs_n = 1'b1;
    bus.data = 16'h0000;
    #20;               // negedge at 20: start a frame
    bus.cs_n = 1'b0;
    bus.data = 16'hf000;  // first beat, sampled at posedge 25
    #10 bus.data = 16'h0001;  // second beat, sampled at posedge 35
    #10 bus.data = 16'h0002;
    #10 bus.cs_n = 1'b1;
  end

  initial begin
    monitor_c m = new();
    m.vif = bus;
    m.run();
    $finish;
  end
endmodule
