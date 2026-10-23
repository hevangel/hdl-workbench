import uvm_pkg::*;
`include "uvm_macros.svh"

interface bus_if(input logic clk);
  logic        req;
  logic [31:0] wdata;
endinterface

class bus_txn extends uvm_sequence_item;
  rand bit [31:0] data;
  `uvm_object_utils(bus_txn)
  function new(string name = "bus_txn"); super.new(name); endfunction
endclass

class bus_driver extends uvm_driver #(bus_txn);
  `uvm_component_utils(bus_driver)
  virtual bus_if vif;
  int driven = 0;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    if (!uvm_config_db#(virtual bus_if)::get(this, "", "vif", vif))
      `uvm_fatal("NOVIF", "bus_if not set")
  endfunction
  task run_phase(uvm_phase phase);
    bus_txn tr;
    $display("DRV: run_phase entered @%0t", $time);
    forever begin
      $display("DRV: calling get_next_item @%0t", $time);
      seq_item_port.get_next_item(tr);
      $display("DRV: got item data=%h at %0t", tr.data, $time);
      @(posedge vif.clk);
      vif.wdata <= tr.data;
      driven++;
      seq_item_port.item_done();
    end
  endtask
endclass

class three_writes_seq extends uvm_sequence #(bus_txn);
  `uvm_object_utils(three_writes_seq)
  function new(string name = "three_writes_seq"); super.new(name); endfunction
  task body();
    bus_txn tr;
    for (int k = 0; k < 3; k++) begin
      tr = bus_txn::type_id::create($sformatf("tr%0d", k));
      $display("SEQ: before start_item %0d @%0t", k, $time);
      start_item(tr);
      $display("SEQ: after start_item %0d @%0t", k, $time);
      tr.data = 32'h100 + k;
      finish_item(tr);
      $display("SEQ: after finish_item %0d @%0t", k, $time);
    end
  endtask
endclass

class seq_test extends uvm_test;
  `uvm_component_utils(seq_test)
  bus_driver drv;
  uvm_sequencer #(bus_txn) sqr;
  function new(string name = "seq_test", uvm_component parent = null); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    drv = bus_driver::type_id::create("drv", this);
    sqr = uvm_sequencer#(bus_txn)::type_id::create("sqr", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    drv.seq_item_port.connect(sqr.seq_item_export);
  endfunction
  task run_phase(uvm_phase phase);
    three_writes_seq seq;
    phase.raise_objection(this);
    seq = three_writes_seq::type_id::create("seq");
    $display("TEST: before seq.start @%0t", $time);
    seq.start(sqr);
    $display("TEST: after seq.start @%0t", $time);
    #10;
    if (drv.driven == 3) $display("TEST_PASS");
    else $display("TEST_FAIL driven=%0d", drv.driven);
    phase.drop_objection(this);
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bif(clk);
  initial begin
    uvm_config_db#(virtual bus_if)::set(null, "*", "vif", bif);
    run_test("seq_test");
  end
endmodule
