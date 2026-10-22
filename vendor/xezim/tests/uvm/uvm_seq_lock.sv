// Sequencer arbitration with lock (IEEE 1800.2 §15.3.2.4): two sequences
// run in parallel on one sequencer. The one that holds lock() must get all
// of its items through the driver back to back, uninterleaved.
import uvm_pkg::*;
`include "uvm_macros.svh"

class li extends uvm_sequence_item;
  string tag; int unsigned n;
  `uvm_object_utils(li)
  function new(string name = "li"); super.new(name); endfunction
endclass

class lseq extends uvm_sequence #(li);
  `uvm_object_utils(lseq)
  string tag; bit use_lock;
  function new(string name = "lseq"); super.new(name); endfunction
  task body();
    if (use_lock) lock();
    for (int i = 0; i < 3; i++) begin
      li t = li::type_id::create("t");
      start_item(t);
      t.tag = tag; t.n = i;
      finish_item(t);
    end
    if (use_lock) unlock();
  endtask
endclass

class ldrv extends uvm_driver #(li);
  `uvm_component_utils(ldrv)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    forever begin
      seq_item_port.get_next_item(req);
      #1;
      $display("T|%s%0d", req.tag, req.n);
      seq_item_port.item_done();
    end
  endtask
endclass

class ltest extends uvm_test;
  `uvm_component_utils(ltest)
  uvm_sequencer #(li) sqr;
  ldrv d;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    sqr = uvm_sequencer#(li)::type_id::create("sqr", this);
    d = ldrv::type_id::create("d", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    d.seq_item_port.connect(sqr.seq_item_export);
  endfunction
  task run_phase(uvm_phase phase);
    lseq a = lseq::type_id::create("a");
    lseq b = lseq::type_id::create("b");
    phase.raise_objection(this);
    a.tag = "A"; a.use_lock = 0;
    b.tag = "B"; b.use_lock = 1;
    fork
      a.start(sqr);
      begin #1; b.start(sqr); end
    join
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("ltest");
endmodule
