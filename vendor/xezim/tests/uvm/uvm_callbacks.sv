// uvm_callbacks (IEEE 1800.2 §10.7): a callback registered on a component
// instance runs through `uvm_do_callbacks; once deleted it no longer does.
import uvm_pkg::*;
`include "uvm_macros.svh"

class pkt extends uvm_object;
  int unsigned v;
  `uvm_object_utils(pkt)
  function new(string name = "pkt"); super.new(name); endfunction
endclass

typedef class cb_comp;

virtual class comp_cb extends uvm_callback;
  function new(string name = "comp_cb"); super.new(name); endfunction
  pure virtual function void pre_send(cb_comp c, pkt p);
endclass

class add_cb extends comp_cb;
  function new(string name = "add_cb"); super.new(name); endfunction
  function void pre_send(cb_comp c, pkt p); p.v += 1000; endfunction
endclass

class cb_comp extends uvm_component;
  `uvm_component_utils(cb_comp)
  `uvm_register_cb(cb_comp, comp_cb)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function int unsigned send(int unsigned v);
    pkt p = pkt::type_id::create("p");
    p.v = v;
    `uvm_do_callbacks(cb_comp, comp_cb, pre_send(this, p))
    return p.v;
  endfunction
endclass

class cb_test extends uvm_test;
  `uvm_component_utils(cb_test)
  cb_comp c;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    c = cb_comp::type_id::create("c", this);
  endfunction
  task run_phase(uvm_phase phase);
    add_cb cb = new("cb");
    phase.raise_objection(this);
    $display("T|none=%0d", c.send(5));
    uvm_callbacks#(cb_comp, comp_cb)::add(c, cb);
    $display("T|with=%0d", c.send(5));
    uvm_callbacks#(cb_comp, comp_cb)::delete(c, cb);
    $display("T|deleted=%0d", c.send(5));
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("cb_test");
endmodule
