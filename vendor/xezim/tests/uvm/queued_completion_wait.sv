`include "uvm_macros.svh"
import uvm_pkg::*;

class queued_packet extends uvm_sequence_item;
  uvm_event completion;
  `uvm_object_utils(queued_packet)
  function new(string name = "queued_packet");
    super.new(name);
    completion = get_event_pool().get("completion");
  endfunction
  task await_completion(); completion.wait_on(); endtask
endclass

class queued_sequence extends uvm_sequence #(queued_packet);
  static int sent = 0;
  static int returned = 0;
  `uvm_object_utils(queued_sequence)
  function new(string name = "queued_sequence"); super.new(name); endfunction
  task body();
    queued_packet packet;
    queued_packet pending[$];
    `uvm_create(packet)
    start_item(packet);
    finish_item(packet);
    sent++;
    pending.push_back(packet);
    pending[0].await_completion();
    returned++;
  endtask
endclass

class queued_driver extends uvm_driver #(queued_packet);
  `uvm_component_utils(queued_driver)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    queued_packet packet;
    forever seq_item_port.get(packet);
  endtask
endclass

class queued_wait_test extends uvm_test;
  `uvm_component_utils(queued_wait_test)
  uvm_sequencer #(queued_packet) sequencer;
  queued_driver driver;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    sequencer = new("sequencer", this);
    driver = queued_driver::type_id::create("driver", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    driver.seq_item_port.connect(sequencer.seq_item_export);
  endfunction
  task run_phase(uvm_phase phase);
    queued_sequence stream = queued_sequence::type_id::create("stream");
    phase.raise_objection(this);
    fork stream.start(sequencer); join_none
    #5;
    $display("T|queued sent=%0d returned=%0d", queued_sequence::sent, queued_sequence::returned);
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("queued_wait_test");
endmodule
