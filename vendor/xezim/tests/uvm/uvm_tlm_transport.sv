// TLM-1 request/response communication (IEEE 1800.2 §12.2):
//   transport  a uvm_transport_imp reached through a blocking AND a
//              nonblocking transport port: transport() consumes target time,
//              nb_transport() answers in zero time or refuses
//   req_rsp    uvm_tlm_req_rsp_channel between a master port and a slave
//              port: requests pipeline ahead of responses, and the channel's
//              request_ap/response_ap broadcast every item
//   tchan      uvm_tlm_transport_channel: a blocking transport() call
//              completes only once the slave has answered through the
//              channel's slave_export
// Every line is tagged `T|<area>|...` with the simulation time.
import uvm_pkg::*;
`include "uvm_macros.svh"

class tx extends uvm_object;
  `uvm_object_utils(tx)
  int a, b;
  function new(string name = "tx"); super.new(name); endfunction
endclass

class rx extends uvm_object;
  `uvm_object_utils(rx)
  int sum;
  function new(string name = "rx"); super.new(name); endfunction
endclass

function automatic tx mk(int a, int b);
  tx t = new;
  t.a = a; t.b = b;
  return t;
endfunction

// ---------------- transport imp ----------------
class adder extends uvm_component;
  `uvm_component_utils(adder)
  uvm_transport_imp #(tx, rx, adder) imp;
  bit busy;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    imp = new("imp", this);
  endfunction
  task transport(tx req, output rx rsp);
    busy = 1;
    #3;
    rsp = new;
    rsp.sum = req.a + req.b;
    busy = 0;
  endtask
  function bit nb_transport(tx req, output rx rsp);
    if (busy) return 0;
    rsp = new;
    rsp.sum = req.a * req.b;
    return 1;
  endfunction
endclass

// ---------------- req/rsp channel ends ----------------
class master extends uvm_component;
  `uvm_component_utils(master)
  uvm_blocking_master_port #(tx, rx) port;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    port = new("port", this);
  endfunction
endclass

class slave extends uvm_component;
  `uvm_component_utils(slave)
  uvm_blocking_slave_port #(tx, rx) port;
  string tag;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    port = new("port", this);
  endfunction
  task run_phase(uvm_phase phase);
    tx req; rx rsp;
    forever begin
      port.peek(req);
      $display("T|%s|slave peek a=%0d t=%0t", tag, req.a, $time);
      port.get(req);
      #2;
      rsp = new;
      rsp.sum = req.a + req.b;
      port.put(rsp);
      $display("T|%s|slave put sum=%0d t=%0t", tag, rsp.sum, $time);
    end
  endtask
endclass

`uvm_analysis_imp_decl(_req)
`uvm_analysis_imp_decl(_rsp)
class chan_mon extends uvm_component;
  `uvm_component_utils(chan_mon)
  uvm_analysis_imp_req #(tx, chan_mon) req_in;
  uvm_analysis_imp_rsp #(rx, chan_mon) rsp_in;
  int n_req, n_rsp;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    req_in = new("req_in", this);
    rsp_in = new("rsp_in", this);
  endfunction
  function void write_req(tx t); n_req++; endfunction
  function void write_rsp(rx r); n_rsp++; endfunction
endclass

class tlm_xport_test extends uvm_test;
  `uvm_component_utils(tlm_xport_test)
  adder add;
  uvm_blocking_transport_port #(tx, rx) bport;
  uvm_nonblocking_transport_port #(tx, rx) nbport;
  uvm_tlm_req_rsp_channel #(tx, rx) chan;
  master m;
  slave s;
  chan_mon cm;
  uvm_tlm_transport_channel #(tx, rx) tchan;
  uvm_blocking_transport_port #(tx, rx) tport;
  slave ts;

  function new(string name, uvm_component parent); super.new(name, parent); endfunction

  function void build_phase(uvm_phase phase);
    add = adder::type_id::create("add", this);
    bport = new("bport", this);
    nbport = new("nbport", this);
    chan = new("chan", this);
    m = master::type_id::create("m", this);
    s = slave::type_id::create("s", this);
    s.tag = "req_rsp";
    cm = chan_mon::type_id::create("cm", this);
    tchan = new("tchan", this);
    tport = new("tport", this);
    ts = slave::type_id::create("ts", this);
    ts.tag = "tchan";
  endfunction

  function void connect_phase(uvm_phase phase);
    bport.connect(add.imp);
    nbport.connect(add.imp);
    m.port.connect(chan.master_export);
    s.port.connect(chan.slave_export);
    chan.request_ap.connect(cm.req_in);
    chan.response_ap.connect(cm.rsp_in);
    tport.connect(tchan.transport_export);
    ts.port.connect(tchan.slave_export);
  endfunction

  task run_phase(uvm_phase phase);
    rx r;
    bit ok;
    phase.raise_objection(this);

    // transport: blocking call takes the target's 3 time units; a
    // nonblocking call while the target is busy is refused
    fork
      begin
        bport.transport(mk(2, 5), r);
        $display("T|transport|b sum=%0d t=%0t", r.sum, $time);
      end
      begin
        #1;
        ok = nbport.nb_transport(mk(3, 4), r);
        $display("T|transport|nb busy ok=%0d t=%0t", ok, $time);
      end
    join
    ok = nbport.nb_transport(mk(3, 4), r);
    $display("T|transport|nb idle ok=%0d product=%0d t=%0t", ok, r.sum, $time);

    // req_rsp: three requests go in back to back, the responses follow at
    // the slave's pace
    for (int i = 1; i <= 3; i++) m.port.put(mk(10 * i, i));
    $display("T|req_rsp|3 requests put t=%0t", $time);
    for (int i = 1; i <= 3; i++) begin
      m.port.get(r);
      $display("T|req_rsp|master got sum=%0d t=%0t", r.sum, $time);
    end
    $display("T|req_rsp|request_ap=%0d response_ap=%0d", cm.n_req, cm.n_rsp);

    // transport channel: one blocking call spans the slave's round trip
    tport.transport(mk(100, 1), r);
    $display("T|tchan|transport sum=%0d t=%0t", r.sum, $time);

    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("tlm_xport_test");
endmodule
