// Phase ORDER across a component hierarchy (IEEE 1800.2 §9.8). build_phase
// and final_phase are TOP-DOWN (uvm_topdown_phase); connect, end_of_elaboration,
// start_of_simulation, extract, check and report are BOTTOM-UP. Siblings run in
// NAME order, not creation order — `mon` is created before `drv` on purpose.
// Each component logs one tagged line per phase.
import uvm_pkg::*;
`include "uvm_macros.svh"

class logc extends uvm_component;
  `uvm_component_utils(logc)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);              $display("T|build|%s", get_name()); endfunction
  function void connect_phase(uvm_phase phase);            $display("T|connect|%s", get_name()); endfunction
  function void end_of_elaboration_phase(uvm_phase phase); $display("T|eoe|%s", get_name()); endfunction
  function void start_of_simulation_phase(uvm_phase phase);$display("T|sos|%s", get_name()); endfunction
  function void extract_phase(uvm_phase phase);            $display("T|extract|%s", get_name()); endfunction
  function void check_phase(uvm_phase phase);              $display("T|check|%s", get_name()); endfunction
  function void report_phase(uvm_phase phase);             $display("T|report|%s", get_name()); endfunction
  function void final_phase(uvm_phase phase);              $display("T|final|%s", get_name()); endfunction
endclass

class agent extends logc;
  `uvm_component_utils(agent)
  logc drv, mon;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    super.build_phase(phase);
    mon = logc::type_id::create("mon", this);   // created before drv on purpose:
    drv = logc::type_id::create("drv", this);   // order must follow NAME, not creation
  endfunction
endclass

class env extends logc;
  `uvm_component_utils(env)
  agent agt;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    super.build_phase(phase);
    agt = agent::type_id::create("agt", this);
  endfunction
endclass

class phase_test extends logc;
  `uvm_component_utils(phase_test)
  env e;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    super.build_phase(phase);
    e = env::type_id::create("e", this);
  endfunction
  task run_phase(uvm_phase phase);
    phase.raise_objection(this);
    $display("T|run|%s", get_name());
    #10;
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("phase_test");
endmodule
