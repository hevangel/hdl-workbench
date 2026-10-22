// Report filtering and actions (IEEE 1800.2 §6.3/§6.4):
//   * default verbosity UVM_MEDIUM: LOW and MEDIUM print, HIGH and DEBUG do not;
//   * raising the component's verbosity to UVM_HIGH lets HIGH through;
//   * an ID set to UVM_NO_ACTION is neither displayed nor counted;
//   * a severity override demotes an ERROR to a WARNING, and it is COUNTED
//     as a warning.
import uvm_pkg::*;
`include "uvm_macros.svh"

class report_test extends uvm_test;
  `uvm_component_utils(report_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    uvm_report_server srv = uvm_report_server::get_server();
    phase.raise_objection(this);
    `uvm_info("V", "at LOW",    UVM_LOW)
    `uvm_info("V", "at MEDIUM", UVM_MEDIUM)
    `uvm_info("V", "at HIGH",   UVM_HIGH)
    `uvm_info("V", "at DEBUG",  UVM_DEBUG)
    set_report_verbosity_level(UVM_HIGH);
    `uvm_info("V", "HIGH after raise", UVM_HIGH)
    set_report_id_action("NOISY", UVM_NO_ACTION);
    `uvm_error("NOISY", "must be silent and uncounted")
    set_report_severity_id_override(UVM_ERROR, "SOFT", UVM_WARNING);
    `uvm_error("SOFT", "demoted to a warning")
    $display("T|err=%0d warn=%0d",
             srv.get_severity_count(UVM_ERROR), srv.get_severity_count(UVM_WARNING));
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("report_test");
endmodule
