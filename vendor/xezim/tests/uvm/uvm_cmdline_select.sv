// run_test() with no argument takes the test from +UVM_TESTNAME, and
// +UVM_VERBOSITY sets the initial verbosity (IEEE 1800.2 §G.1).
import uvm_pkg::*;
`include "uvm_macros.svh"

class test_a extends uvm_test;
  `uvm_component_utils(test_a)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    phase.raise_objection(this); $display("T|ran|test_a"); phase.drop_objection(this);
  endtask
endclass

class test_b extends uvm_test;
  `uvm_component_utils(test_b)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    phase.raise_objection(this);
    $display("T|ran|test_b");
    `uvm_info("VB", "HIGH message", UVM_HIGH)
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test();
endmodule
