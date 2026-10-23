// uvm_root::set_timeout (IEEE 1800.2 §F.7): a run_phase that never drops
// its objection is ended by the global timeout with a PH_TIMEOUT fatal at
// exactly the configured time, not by the simulator's own max time.
import uvm_pkg::*;
`include "uvm_macros.svh"

class hang_test extends uvm_test;
  `uvm_component_utils(hang_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    uvm_root::get().set_timeout(300, 1);
  endfunction
  task run_phase(uvm_phase phase);
    phase.raise_objection(this);
    forever #10;
  endtask
endclass

module top;
  initial run_test("hang_test");
endmodule
