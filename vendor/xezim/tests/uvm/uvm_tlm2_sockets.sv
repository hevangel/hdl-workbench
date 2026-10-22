// TLM-2 sockets and the generic payload (IEEE 1800.2 §12.3):
//   b   blocking transport crossing two hierarchy levels on each side
//       through passthrough sockets (initiator -> ienv passthrough ->
//       tenv passthrough -> target). The target is a 16-byte memory: it
//       consumes 2 time units, annotates 5ns on the delay argument and
//       answers an out-of-range address with ADDRESS_ERROR.
//   nb  nonblocking transport, both directions: the forward call returns
//       ACCEPTED and the target answers later on the backward path
//       (BEGIN_RESP, completed by the initiator); UPDATED moves the phase
//       to END_REQ and annotates delay; COMPLETED finishes at once
import uvm_pkg::*;
`include "uvm_macros.svh"

typedef uvm_tlm_generic_payload gp_t;

function automatic gp_t mk_gp(uvm_tlm_command_e cmd, bit [63:0] addr, int n, byte unsigned first);
  gp_t gp = new("gp");
  byte unsigned d[];
  d = new[n];
  foreach (d[i]) d[i] = first + i;
  gp.set_command(cmd);
  gp.set_address(addr);
  gp.set_data(d);
  gp.set_data_length(n);
  gp.set_response_status(UVM_TLM_INCOMPLETE_RESPONSE);
  return gp;
endfunction

function automatic string bytes(gp_t gp);
  byte unsigned d[];
  string s = "";
  gp.get_data(d);
  for (int i = 0; i < gp.get_data_length(); i++) s = {s, $sformatf("%02h", d[i])};
  return s;
endfunction

// ---------------- blocking ----------------
class mem_target extends uvm_component;
  `uvm_component_utils(mem_target)
  uvm_tlm_b_target_socket #(mem_target) sock;
  byte unsigned mem[16];
  int n;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    sock = new("sock", this);
  endfunction
  task b_transport(gp_t gp, uvm_tlm_time delay);
    byte unsigned d[];
    int a = gp.get_address();
    n++;
    #2;
    delay.incr(5, 1ns);
    if (a + gp.get_data_length() > 16) begin
      gp.set_response_status(UVM_TLM_ADDRESS_ERROR_RESPONSE);
      return;
    end
    gp.get_data(d);
    for (int i = 0; i < gp.get_data_length(); i++)
      if (gp.is_write()) mem[a + i] = d[i];
      else d[i] = mem[a + i];
    if (gp.is_read()) gp.set_data(d);
    gp.set_response_status(UVM_TLM_OK_RESPONSE);
  endtask
endclass

class b_init extends uvm_component;
  `uvm_component_utils(b_init)
  uvm_tlm_b_initiator_socket #() sock;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    sock = new("sock", this);
  endfunction
endclass

class ienv extends uvm_component;
  `uvm_component_utils(ienv)
  b_init ini;
  uvm_tlm_b_passthrough_initiator_socket #() pt;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    pt = new("pt", this);
  endfunction
  function void build_phase(uvm_phase phase);
    ini = b_init::type_id::create("ini", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    ini.sock.connect(pt);
  endfunction
endclass

class tenv extends uvm_component;
  `uvm_component_utils(tenv)
  mem_target tgt;
  uvm_tlm_b_passthrough_target_socket #() pt;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    pt = new("pt", this);
  endfunction
  function void build_phase(uvm_phase phase);
    tgt = mem_target::type_id::create("tgt", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    pt.connect(tgt.sock);
  endfunction
endclass

// ---------------- nonblocking ----------------
class nb_tgt extends uvm_component;
  `uvm_component_utils(nb_tgt)
  uvm_tlm_nb_target_socket #(nb_tgt, gp_t, uvm_tlm_phase_e) sock;
  gp_t pending[$];
  event got_req;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    sock = new("sock", this);
  endfunction
  // the address picks the target's answer
  function uvm_tlm_sync_e nb_transport_fw(gp_t gp, ref uvm_tlm_phase_e p, input uvm_tlm_time delay);
    $display("T|nb|fw addr=%0h phase=%s t=%0t", gp.get_address(), p.name(), $time);
    case (gp.get_address())
      'h10: begin
        pending.push_back(gp);
        ->got_req;
        return UVM_TLM_ACCEPTED;
      end
      'h20: begin
        p = END_REQ;
        delay.incr(7, 1ns);
        return UVM_TLM_UPDATED;
      end
      default: begin
        gp.set_response_status(UVM_TLM_OK_RESPONSE);
        return UVM_TLM_COMPLETED;
      end
    endcase
  endfunction
  task run_phase(uvm_phase phase);
    forever begin
      uvm_tlm_phase_e p = BEGIN_RESP;
      uvm_tlm_time d = new("d");
      uvm_tlm_sync_e s;
      gp_t gp;
      wait (pending.size() > 0);
      gp = pending.pop_front();
      #3;
      gp.set_response_status(UVM_TLM_OK_RESPONSE);
      s = sock.nb_transport_bw(gp, p, d);
      $display("T|nb|bw returned %s phase=%s t=%0t", s.name(), p.name(), $time);
    end
  endtask
endclass

class nb_init extends uvm_component;
  `uvm_component_utils(nb_init)
  uvm_tlm_nb_initiator_socket #(nb_init, gp_t, uvm_tlm_phase_e) sock;
  event resp;
  function new(string name, uvm_component parent);
    super.new(name, parent);
    sock = new("sock", this);
  endfunction
  function uvm_tlm_sync_e nb_transport_bw(gp_t gp, ref uvm_tlm_phase_e p, input uvm_tlm_time delay);
    $display("T|nb|bw addr=%0h phase=%s resp=%s t=%0t", gp.get_address(), p.name(),
             gp.get_response_string(), $time);
    p = END_RESP;
    ->resp;
    return UVM_TLM_COMPLETED;
  endfunction
endclass

class tlm2_test extends uvm_test;
  `uvm_component_utils(tlm2_test)
  ienv ie;
  tenv te;
  nb_init ni;
  nb_tgt nt;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    ie = ienv::type_id::create("ie", this);
    te = tenv::type_id::create("te", this);
    ni = nb_init::type_id::create("ni", this);
    nt = nb_tgt::type_id::create("nt", this);
  endfunction
  function void connect_phase(uvm_phase phase);
    ie.pt.connect(te.pt);
    ni.sock.connect(nt.sock);
  endfunction

  task run_phase(uvm_phase phase);
    gp_t gp;
    uvm_tlm_time delay;
    uvm_tlm_phase_e p;
    uvm_tlm_sync_e s;
    phase.raise_objection(this);

    delay = new("delay");
    gp = mk_gp(UVM_TLM_WRITE_COMMAND, 4, 4, 8'hA0);
    ie.ini.sock.b_transport(gp, delay);
    $display("T|b|write resp=%s delay=%0.1f t=%0t", gp.get_response_string(),
             delay.get_realtime(1ns), $time);
    gp = mk_gp(UVM_TLM_READ_COMMAND, 2, 8, 0);
    ie.ini.sock.b_transport(gp, delay);
    $display("T|b|read resp=%s ok=%0d data=%s delay=%0.1f t=%0t", gp.get_response_string(),
             gp.is_response_ok(), bytes(gp), delay.get_realtime(1ns), $time);
    gp = mk_gp(UVM_TLM_WRITE_COMMAND, 14, 4, 0);
    ie.ini.sock.b_transport(gp, delay);
    $display("T|b|overrun resp=%s ok=%0d calls=%0d", gp.get_response_string(),
             gp.is_response_ok(), te.tgt.n);

    delay = new("delay");
    gp = mk_gp(UVM_TLM_WRITE_COMMAND, 'h10, 1, 0);
    p = BEGIN_REQ;
    s = ni.sock.nb_transport_fw(gp, p, delay);
    $display("T|nb|fw returned %s phase=%s t=%0t", s.name(), p.name(), $time);
    @(ni.resp);
    #1;
    gp = mk_gp(UVM_TLM_WRITE_COMMAND, 'h20, 1, 0);
    p = BEGIN_REQ;
    s = ni.sock.nb_transport_fw(gp, p, delay);
    $display("T|nb|fw returned %s phase=%s delay=%0.1f t=%0t", s.name(), p.name(),
             delay.get_realtime(1ns), $time);
    gp = mk_gp(UVM_TLM_READ_COMMAND, 'h30, 1, 0);
    p = BEGIN_REQ;
    s = ni.sock.nb_transport_fw(gp, p, delay);
    $display("T|nb|fw returned %s resp=%s t=%0t", s.name(), gp.get_response_string(), $time);

    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("tlm2_test");
endmodule
