// TLM-1 connection topology (IEEE 1800.2 §5.5, §12.2.5):
//   hier    a put port climbs out of its agent through the agent's own port,
//           crosses to an env's export and down to the imp; every hop
//           resolves to the one imp (size() and get_if())
//   multi   a port bound to two imps (min 2, max 2): set_if() picks which
//           one a put reaches; the list is ordered by imp full name
//   fanout  one analysis port reaching three subscribers directly and two
//           more through an env's analysis export: size() is 5 and each
//           write reaches all five; a port with no subscribers is legal
//   fifo_ap uvm_tlm_fifo's put_ap and get_ap: put/try_put and get/try_get
//           publish, peek/try_peek do not
import uvm_pkg::*;
`include "uvm_macros.svh"

class sink extends uvm_component;
  `uvm_component_utils(sink)
  uvm_blocking_put_imp #(int, sink) imp;
  int got[$];
  function new(string name, uvm_component parent);
    super.new(name, parent);
    imp = new("imp", this);
  endfunction
  task put(int t);
    #1 got.push_back(t);
  endtask
endclass

class producer extends uvm_component;
  `uvm_component_utils(producer)
  uvm_blocking_put_port #(int) port;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    port = new("port", this);
  endfunction
endclass

class agent extends uvm_component;
  `uvm_component_utils(agent)
  producer prod;
  uvm_blocking_put_port #(int) port;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    port = new("port", this);
  endfunction
  function void build_phase(uvm_phase phase);
    prod = producer::type_id::create("prod", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    prod.port.connect(port);
  endfunction
endclass

class sink_env extends uvm_component;
  `uvm_component_utils(sink_env)
  sink leaf;
  uvm_blocking_put_export #(int) exp;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    exp = new("exp", this);
  endfunction
  function void build_phase(uvm_phase phase);
    leaf = sink::type_id::create("leaf", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    exp.connect(leaf.imp);
  endfunction
endclass

class counter extends uvm_subscriber #(int);
  `uvm_component_utils(counter)
  int n, last;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void write(int t);
    n++;
    last = t;
  endfunction
endclass

class sub_env extends uvm_component;
  `uvm_component_utils(sub_env)
  uvm_analysis_export #(int) analysis_export;
  counter c[2];
  function new(string name, uvm_component parent);
    super.new(name, parent);
    analysis_export = new("analysis_export", this);
  endfunction
  function void build_phase(uvm_phase phase);
    foreach (c[i]) c[i] = counter::type_id::create($sformatf("c%0d", i), this);
  endfunction
  function void connect_phase(uvm_phase phase);
    foreach (c[i]) analysis_export.connect(c[i].analysis_export);
  endfunction
endclass

`uvm_analysis_imp_decl(_put)
`uvm_analysis_imp_decl(_get)
class fifo_watch extends uvm_component;
  `uvm_component_utils(fifo_watch)
  uvm_analysis_imp_put #(int, fifo_watch) put_in;
  uvm_analysis_imp_get #(int, fifo_watch) get_in;
  string log;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    put_in = new("put_in", this);
    get_in = new("get_in", this);
  endfunction
  function void write_put(int t); log = {log, $sformatf(" P%0d", t)}; endfunction
  function void write_get(int t); log = {log, $sformatf(" G%0d", t)}; endfunction
endclass

class tlm_connect_test extends uvm_test;
  `uvm_component_utils(tlm_connect_test)
  agent agt;
  sink_env senv;
  uvm_blocking_put_port #(int) mport;
  sink ma, mb;
  uvm_analysis_port #(int) ap, lonely_ap;
  counter direct[3];
  sub_env subs;
  uvm_tlm_fifo #(int) fifo;
  fifo_watch fw;

  function new(string name, uvm_component parent); super.new(name, parent); endfunction

  function void build_phase(uvm_phase phase);
    agt = agent::type_id::create("agt", this);
    senv = sink_env::type_id::create("senv", this);
    mport = new("mport", this, 2, 2);
    // created in reverse name order: the imp list is sorted by name anyway
    mb = sink::type_id::create("mb", this);
    ma = sink::type_id::create("ma", this);
    ap = new("ap", this);
    lonely_ap = new("lonely_ap", this);
    foreach (direct[i]) direct[i] = counter::type_id::create($sformatf("d%0d", i), this);
    subs = sub_env::type_id::create("subs", this);
    fifo = new("fifo", this, 4);
    fw = fifo_watch::type_id::create("fw", this);
  endfunction

  function void connect_phase(uvm_phase phase);
    agt.port.connect(senv.exp);
    mport.connect(mb.imp);
    mport.connect(ma.imp);
    foreach (direct[i]) ap.connect(direct[i].analysis_export);
    ap.connect(subs.analysis_export);
    fifo.put_ap.connect(fw.put_in);
    fifo.get_ap.connect(fw.get_in);
  endfunction

  function void end_of_elaboration_phase(uvm_phase phase);
    $display("T|hier|prod.port size=%0d agt.port size=%0d senv.exp size=%0d",
             agt.prod.port.size(), agt.port.size(), senv.exp.size());
    $display("T|hier|resolves to %s", agt.prod.port.get_if(0).get_full_name());
    $display("T|multi|size=%0d if0=%s if1=%s", mport.size(),
             mport.get_if(0).get_full_name(), mport.get_if(1).get_full_name());
    $display("T|fanout|ap size=%0d lonely size=%0d", ap.size(), lonely_ap.size());
  endfunction

  task run_phase(uvm_phase phase);
    int v;
    bit ok;
    phase.raise_objection(this);

    for (int i = 1; i <= 3; i++) agt.prod.port.put(i * 11);
    $display("T|hier|leaf got %p t=%0t", senv.leaf.got, $time);

    mport.put(1);           // default: index 0 = ma
    mport.set_if(1);
    mport.put(2);           // mb
    mport.put(3);           // mb
    mport.set_if(0);
    mport.put(4);           // ma
    $display("T|multi|ma=%p mb=%p", ma.got, mb.got);

    ap.write(7);
    ap.write(8);
    lonely_ap.write(9);
    $display("T|fanout|d=%0d,%0d,%0d subs=%0d,%0d last=%0d,%0d,%0d,%0d,%0d",
             direct[0].n, direct[1].n, direct[2].n, subs.c[0].n, subs.c[1].n,
             direct[0].last, direct[1].last, direct[2].last, subs.c[0].last, subs.c[1].last);

    fifo.put(1);
    ok = fifo.try_put(2);
    fifo.put(3);
    fifo.peek(v);
    ok = fifo.try_peek(v);
    fifo.get(v);
    ok = fifo.try_get(v);
    $display("T|fifo_ap|log=%s used=%0d", fw.log, fifo.used());

    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("tlm_connect_test");
endmodule
