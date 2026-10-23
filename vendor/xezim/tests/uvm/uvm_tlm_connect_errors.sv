// TLM-1 connection checking (IEEE 1800.2 §5.5): every misuse below is a
// UVM_ERROR with ID "Connection Error", reported by the offending port.
//   bad_imp      imp.connect(...)           an imp's provider is fixed
//   bad_exp      export.connect(port)       exports cannot drive ports
//   unconnected  a required port (min 1) left open: caught at
//                end_of_elaboration
//   over         a max-1 port bound to two imps: caught at end_of_elaboration
//   optional     a min-0 port left open is legal: no error
// The connect-time checks return and connect_phase goes on. The size checks
// run as end_of_elaboration starts (uvm_root::phase_started), and with any
// error on record by then uvm_root stops the run: UVM_FATAL BUILDERR, so
// end_of_elaboration_phase and run_phase never execute.
import uvm_pkg::*;
`include "uvm_macros.svh"

class sink extends uvm_component;
  `uvm_component_utils(sink)
  uvm_blocking_put_imp #(int, sink) imp;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    imp = new("imp", this);
  endfunction
  task put(int t); endtask
endclass

class holder extends uvm_component;
  `uvm_component_utils(holder)
  uvm_blocking_put_port #(int) port;
  uvm_blocking_put_export #(int) exp;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    port = new("port", this);
    exp = new("exp", this);
  endfunction
endclass

class tlm_err_test extends uvm_test;
  `uvm_component_utils(tlm_err_test)
  sink s1, s2;
  holder bad_imp, bad_exp, unconnected, over, optional;
  uvm_blocking_put_port #(int) opt_port;

  function new(string name, uvm_component parent); super.new(name, parent); endfunction

  function void build_phase(uvm_phase phase);
    s1 = sink::type_id::create("s1", this);
    s2 = sink::type_id::create("s2", this);
    bad_imp = holder::type_id::create("bad_imp", this);
    bad_exp = holder::type_id::create("bad_exp", this);
    unconnected = holder::type_id::create("unconnected", this);
    over = holder::type_id::create("over", this);
    optional = holder::type_id::create("optional", this);
    opt_port = new("opt_port", optional, 0, 1);
  endfunction

  function void connect_phase(uvm_phase phase);
    // every holder's export is satisfied, so only the intended checks fire
    bad_imp.exp.connect(s1.imp);
    bad_exp.exp.connect(s1.imp);
    unconnected.exp.connect(s1.imp);
    over.exp.connect(s1.imp);
    optional.exp.connect(s1.imp);
    bad_imp.port.connect(s1.imp);
    bad_exp.port.connect(s1.imp);
    optional.port.connect(s1.imp);

    s1.imp.connect(s2.imp);
    $display("T|connect|imp.connect returned");
    bad_exp.exp.connect(bad_exp.port);
    $display("T|connect|export.connect(port) returned");
    over.port.connect(s1.imp);
    over.port.connect(s2.imp);
  endfunction

  function void end_of_elaboration_phase(uvm_phase phase);
    $display("T|eoe|reached");
  endfunction

  task run_phase(uvm_phase phase);
    $display("T|run|reached");
  endtask
endclass

module top;
  initial run_test("tlm_err_test");
endmodule
