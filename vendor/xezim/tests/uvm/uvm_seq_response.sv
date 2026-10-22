// Request/response (IEEE 1800.2 §15.4): the driver answers each request with
// a response carrying set_id_info(req), and the sequence's get_response()
// must receive the response matching the request it just sent.
import uvm_pkg::*;
`include "uvm_macros.svh"

class rr extends uvm_sequence_item;
  int unsigned v;
  `uvm_object_utils(rr)
  function new(string name = "rr"); super.new(name); endfunction
endclass

class rr_seq extends uvm_sequence #(rr);
  `uvm_object_utils(rr_seq)
  function new(string name = "rr_seq"); super.new(name); endfunction
  task body();
    for (int i = 1; i <= 4; i++) begin
      rr q = rr::type_id::create("q");
      start_item(q);
      q.v = i;
      finish_item(q);
      get_response(rsp);
      $display("T|req=%0d rsp=%0d match=%0d", q.v, rsp.v,
               rsp.get_transaction_id() == q.get_transaction_id());
    end
  endtask
endclass

class rr_drv extends uvm_driver #(rr);
  `uvm_component_utils(rr_drv)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    forever begin
      rr r;
      seq_item_port.get_next_item(req);
      #2;
      r = rr::type_id::create("r");
      r.set_id_info(req);
      r.v = req.v * 10;
      seq_item_port.item_done();
      seq_item_port.put_response(r);
    end
  endtask
endclass

class rr_test extends uvm_test;
  `uvm_component_utils(rr_test)
  uvm_sequencer #(rr) sqr;
  rr_drv d;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    sqr = uvm_sequencer#(rr)::type_id::create("sqr", this);
    d = rr_drv::type_id::create("d", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    d.seq_item_port.connect(sqr.seq_item_export);
  endfunction
  task run_phase(uvm_phase phase);
    rr_seq s = rr_seq::type_id::create("s");
    phase.raise_objection(this);
    s.start(sqr);
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("rr_test");
endmodule
