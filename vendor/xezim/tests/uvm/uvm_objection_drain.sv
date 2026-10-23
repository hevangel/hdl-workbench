// run_phase ends when the LAST objection drops plus the drain time
// (IEEE 1800.2 §10.5.1). Two components hold for 30 and 70; the test
// sets a drain time of 25, so the phase must end at t=95.
import uvm_pkg::*;
`include "uvm_macros.svh"

class holder extends uvm_component;
  `uvm_component_utils(holder)
  int unsigned hold = 0;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    phase.raise_objection(this);
    #(hold);
    $display("T|drop|%s|%0t", get_name(), $time);
    phase.drop_objection(this);
  endtask
endclass

class drain_test extends uvm_test;
  `uvm_component_utils(drain_test)
  holder a, b;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    a = holder::type_id::create("a", this); a.hold = 30;
    b = holder::type_id::create("b", this); b.hold = 70;
  endfunction
  task run_phase(uvm_phase phase);
    uvm_objection obj = phase.get_objection();
    phase.raise_objection(this);
    obj.set_drain_time(this, 25);
    phase.drop_objection(this);
  endtask
  function void extract_phase(uvm_phase phase);
    $display("T|extract_at|%0t", $time);
  endfunction
endclass

module top;
  initial run_test("drain_test");
endmodule
