// A virtual sequence coordinating two agents (IEEE 1800.2 §14): it holds
// handles to both sequencers and starts a sub-sequence on each in parallel.
// Each driver tags the items it receives; both must see their full share.
import uvm_pkg::*;
`include "uvm_macros.svh"

class item extends uvm_sequence_item;
  rand int unsigned v;
  `uvm_object_utils(item)
  function new(string name = "item"); super.new(name); endfunction
endclass

class burst extends uvm_sequence #(item);
  `uvm_object_utils(burst)
  int unsigned n = 0, base = 0;
  function new(string name = "burst"); super.new(name); endfunction
  task body();
    for (int i = 0; i < n; i++) begin
      item t = item::type_id::create("t");
      start_item(t);
      t.v = base + i;
      finish_item(t);
    end
  endtask
endclass

class drv extends uvm_driver #(item);
  `uvm_component_utils(drv)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    forever begin
      seq_item_port.get_next_item(req);
      #1;
      $display("T|%s|%0d", get_name(), req.v);
      seq_item_port.item_done();
    end
  endtask
endclass

class vseq extends uvm_sequence;
  `uvm_object_utils(vseq)
  uvm_sequencer #(item) sa, sb;
  function new(string name = "vseq"); super.new(name); endfunction
  task body();
    burst ba = burst::type_id::create("ba");
    burst bb = burst::type_id::create("bb");
    ba.n = 3; ba.base = 100;
    bb.n = 4; bb.base = 200;
    fork
      ba.start(sa);
      bb.start(sb);
    join
    $display("T|vseq_done|%0t", $time);
  endtask
endclass

class vtest extends uvm_test;
  `uvm_component_utils(vtest)
  uvm_sequencer #(item) sqa, sqb;
  drv da, db;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    sqa = uvm_sequencer#(item)::type_id::create("sqa", this);
    sqb = uvm_sequencer#(item)::type_id::create("sqb", this);
    da = drv::type_id::create("da", this);
    db = drv::type_id::create("db", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    da.seq_item_port.connect(sqa.seq_item_export);
    db.seq_item_port.connect(sqb.seq_item_export);
  endfunction
  task run_phase(uvm_phase phase);
    vseq v = vseq::type_id::create("v");
    phase.raise_objection(this);
    v.sa = sqa; v.sb = sqb;
    v.start(null);
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("vtest");
endmodule
