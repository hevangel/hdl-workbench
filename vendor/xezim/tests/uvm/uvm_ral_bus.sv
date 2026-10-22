// RAL against a DUT on a simple bus. The register file implements the access
// policies its model declares, so frontdoor traffic, the mirror and the DUT
// must agree. One environment, five tests chosen with +UVM_TESTNAME:
//   ral_fd_test        W1C / RC / RO semantics, set() + needs_update + update()
//   ral_reset_seq_test the built-in uvm_reg_hw_reset_seq on a clean reset
//   ral_mismatch_test  mirror(UVM_CHECK) catching a value changed behind the
//                      model's back, then agreeing once resynchronised
//   ral_pred_test      auto-predict OFF: an explicit uvm_reg_predictor keeps
//                      the mirror current, including for traffic the register
//                      model never issued
//   ral_mem_test       uvm_mem word writes and reads through the map
//
//   offset  register  policy  reset
//   0x00    CTRL      RW      0x000000A5
//   0x04    STATUS    W1C     0          (hardware sets bits)
//   0x08    EVENTS    RC      0          (hardware sets bits, a read clears)
//   0x0C    ID        RO      0xCAFE0001
//   0x10    SCRATCH   RW      0
//   0x40    MEM       8 x 32-bit RW words
import uvm_pkg::*;
`include "uvm_macros.svh"

interface rbus_if(input logic clk);
  logic        req = 0, wr = 0;
  logic [7:0]  addr = 0;
  logic [31:0] wdata = 0, rdata;
  // hardware-side stimulus the testbench drives directly
  logic        hw_pulse = 0, hw_ctrl_we = 0;
  logic [31:0] hw_status = 0, hw_events = 0, hw_ctrl = 0;
endinterface

module rdut(rbus_if b);
  logic [31:0] CTRL, STATUS, EVENTS, ID, SCRATCH;
  logic [31:0] MEM [8];
  initial begin
    CTRL = 32'hA5; STATUS = 0; EVENTS = 0; ID = 32'hCAFE_0001; SCRATCH = 0;
    b.rdata = 0;
    foreach (MEM[i]) MEM[i] = 0;
  end
  always @(posedge b.clk) begin
    if (b.hw_pulse) begin
      STATUS <= STATUS | b.hw_status;
      EVENTS <= EVENTS | b.hw_events;
    end
    if (b.hw_ctrl_we) CTRL <= b.hw_ctrl;
    if (b.req) begin
      if (b.wr) begin
        case (b.addr)
          8'h00: CTRL    <= b.wdata;
          8'h04: STATUS  <= STATUS & ~b.wdata;          // W1C
          8'h08: ;                                      // RC: writes ignored
          8'h0C: ;                                      // RO
          8'h10: SCRATCH <= b.wdata;
          default: if (b.addr >= 8'h40 && b.addr < 8'h60) MEM[(b.addr - 8'h40) >> 2] <= b.wdata;
        endcase
      end else begin
        case (b.addr)
          8'h00: b.rdata <= CTRL;
          8'h04: b.rdata <= STATUS;
          8'h08: begin b.rdata <= EVENTS; EVENTS <= 0; end  // read clears
          8'h0C: b.rdata <= ID;
          8'h10: b.rdata <= SCRATCH;
          default: b.rdata <= (b.addr >= 8'h40 && b.addr < 8'h60)
                              ? MEM[(b.addr - 8'h40) >> 2] : 32'hDEAD_BEEF;
        endcase
      end
    end
  end
endmodule

// ---------------- bus agent ----------------
class btxn extends uvm_sequence_item;
  bit write; bit [7:0] addr; bit [31:0] data;
  `uvm_object_utils(btxn)
  function new(string name = "btxn"); super.new(name); endfunction
endclass

class bdrv extends uvm_driver #(btxn);
  `uvm_component_utils(bdrv)
  virtual rbus_if vif;
  int unsigned writes = 0;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    forever begin
      seq_item_port.get_next_item(req);
      @(posedge vif.clk);
      vif.req <= 1; vif.wr <= req.write; vif.addr <= req.addr; vif.wdata <= req.data;
      @(posedge vif.clk);
      vif.req <= 0;
      @(posedge vif.clk);
      if (req.write) writes++;
      else req.data = vif.rdata;
      seq_item_port.item_done();
    end
  endtask
endclass

class bmon extends uvm_monitor;
  `uvm_component_utils(bmon)
  virtual rbus_if vif;
  uvm_analysis_port #(btxn) ap;
  function new(string name, uvm_component parent); super.new(name, parent); ap = new("ap", this); endfunction
  task run_phase(uvm_phase phase);
    forever begin
      @(posedge vif.clk);
      if (vif.req) begin
        btxn t = btxn::type_id::create("t");
        t.write = vif.wr; t.addr = vif.addr; t.data = vif.wdata;
        if (!vif.wr) begin @(posedge vif.clk); t.data = vif.rdata; end
        ap.write(t);
      end
    end
  endtask
endclass

class braw_seq extends uvm_sequence #(btxn);
  `uvm_object_utils(braw_seq)
  bit [7:0] addr; bit [31:0] data;
  function new(string name = "braw_seq"); super.new(name); endfunction
  task body();
    btxn t = btxn::type_id::create("t");
    start_item(t); t.write = 1; t.addr = addr; t.data = data; finish_item(t);
  endtask
endclass

// ---------------- register model ----------------
class r32 extends uvm_reg;
  `uvm_object_utils(r32)
  uvm_reg_field V;
  string acc = "RW"; bit [31:0] rst = 0; bit vol = 0;
  function new(string name = "r32"); super.new(name, 32, UVM_NO_COVERAGE); endfunction
  virtual function void build();
    V = uvm_reg_field::type_id::create("V");
    V.configure(this, 32, 0, acc, vol, rst, 1, 0, 1);
  endfunction
endclass

class rblock extends uvm_reg_block;
  `uvm_object_utils(rblock)
  r32 CTRL, STATUS, EVENTS, ID, SCRATCH;
  uvm_mem MEM;
  function new(string name = "rblock"); super.new(name, UVM_NO_COVERAGE); endfunction
  function r32 mk(string n, string acc, bit [31:0] rst, bit vol, int off);
    r32 r = r32::type_id::create(n);
    r.acc = acc; r.rst = rst; r.vol = vol;
    r.configure(this, null, "");
    r.build();
    default_map.add_reg(r, off, "RW");
    return r;
  endfunction
  virtual function void build();
    default_map = create_map("map", 0, 4, UVM_LITTLE_ENDIAN);
    CTRL    = mk("CTRL",    "RW",  32'hA5,        0, 'h00);
    STATUS  = mk("STATUS",  "W1C", 0,             0, 'h04);
    EVENTS  = mk("EVENTS",  "RC",  0,             1, 'h08);
    ID      = mk("ID",      "RO",  32'hCAFE_0001, 0, 'h0C);
    SCRATCH = mk("SCRATCH", "RW",  0,             0, 'h10);
    MEM = new("MEM", 8, 32, "RW", UVM_NO_COVERAGE);
    MEM.configure(this, "");
    default_map.add_mem(MEM, 'h40, "RW");
    lock_model();
  endfunction
endclass

class badapter extends uvm_reg_adapter;
  `uvm_object_utils(badapter)
  function new(string name = "badapter"); super.new(name); endfunction
  virtual function uvm_sequence_item reg2bus(const ref uvm_reg_bus_op rw);
    btxn t = btxn::type_id::create("t");
    t.write = (rw.kind == UVM_WRITE); t.addr = rw.addr; t.data = rw.data;
    return t;
  endfunction
  virtual function void bus2reg(uvm_sequence_item bus_item, ref uvm_reg_bus_op rw);
    btxn t;
    if (!$cast(t, bus_item)) `uvm_fatal("ADAPT", "not a btxn")
    rw.kind = t.write ? UVM_WRITE : UVM_READ;
    rw.addr = t.addr; rw.data = t.data; rw.status = UVM_IS_OK;
  endfunction
endclass

// ---------------- base test: the environment ----------------
class ral_base extends uvm_test;
  `uvm_component_utils(ral_base)
  uvm_sequencer #(btxn) sqr;
  bdrv drv;
  bmon mon;
  rblock blk;
  badapter adp;
  uvm_reg_predictor #(btxn) prd;
  virtual rbus_if vif;
  bit explicit_predict = 0;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    if (!uvm_config_db#(virtual rbus_if)::get(this, "", "vif", vif)) `uvm_fatal("NOVIF", "no vif")
    sqr = uvm_sequencer#(btxn)::type_id::create("sqr", this);
    drv = bdrv::type_id::create("drv", this);
    mon = bmon::type_id::create("mon", this);
    prd = uvm_reg_predictor#(btxn)::type_id::create("prd", this);
    blk = rblock::type_id::create("blk");
    blk.build();
    blk.reset();   // lock_model() does not; mirror and desired start at the reset values
    adp = badapter::type_id::create("adp");
  endfunction
  function void connect_phase(uvm_phase phase);
    drv.vif = vif; mon.vif = vif;
    drv.seq_item_port.connect(sqr.seq_item_export);
    blk.default_map.set_sequencer(sqr, adp);
    if (explicit_predict) begin
      blk.default_map.set_auto_predict(0);
      prd.map = blk.default_map;
      prd.adapter = adp;
      mon.ap.connect(prd.bus_in);
    end else begin
      blk.default_map.set_auto_predict(1);
    end
  endfunction
  // drive the DUT's hardware-side inputs for one clock
  task hw(bit [31:0] status_set, bit [31:0] events_set);
    @(negedge vif.clk);
    vif.hw_status = status_set; vif.hw_events = events_set; vif.hw_pulse = 1;
    @(negedge vif.clk);
    vif.hw_pulse = 0;
  endtask
  function bit [31:0] mv(uvm_reg r); return r.get_mirrored_value(); endfunction
endclass

class ral_fd_test extends ral_base;
  `uvm_component_utils(ral_fd_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    uvm_status_e st; uvm_reg_data_t d;
    phase.raise_objection(this);
    // desired vs mirrored, on a freshly reset model: set() alone writes
    // nothing; update() writes what differs. A VOLATILE field always needs
    // update (uvm_reg_field::needs_update ORs in m_volatile), so EVENTS keeps
    // the block "dirty" and costs one extra bus write of its own.
    drv.writes = 0;
    blk.SCRATCH.set(32'hBEEF);
    $display("T|upd|before scratch=%0d events=%0d ctrl=%0d bus_writes=%0d mirror=%0h",
             blk.SCRATCH.needs_update(), blk.EVENTS.needs_update(),
             blk.CTRL.needs_update(), drv.writes, mv(blk.SCRATCH));
    blk.update(st);
    $display("T|upd|after scratch=%0d events=%0d block=%0d bus_writes=%0d mirror=%0h",
             blk.SCRATCH.needs_update(), blk.EVENTS.needs_update(),
             blk.needs_update(), drv.writes, mv(blk.SCRATCH));
    // W1C: hardware raises 0x3C; writing 0x0C clears bits 3:2 in DUT and mirror
    hw(32'h3C, 0);
    blk.STATUS.read(st, d);
    blk.STATUS.write(st, 32'h0C);
    blk.STATUS.mirror(st, UVM_CHECK);
    $display("T|w1c|read=%0h mirror=%0h", d, mv(blk.STATUS));
    // RC: hardware raises 0x55; the first read returns it and clears it
    hw(0, 32'h55);
    blk.EVENTS.read(st, d);
    $display("T|rc|first=%0h mirror=%0h", d, mv(blk.EVENTS));
    blk.EVENTS.read(st, d);
    $display("T|rc|second=%0h", d);
    // RO: a write changes neither the DUT nor the mirror
    blk.ID.write(st, 32'h1234_5678);
    blk.ID.read(st, d);
    $display("T|ro|read=%0h mirror=%0h", d, mv(blk.ID));
    blk.SCRATCH.mirror(st, UVM_CHECK);
    phase.drop_objection(this);
  endtask
endclass

class ral_reset_seq_test extends ral_base;
  `uvm_component_utils(ral_reset_seq_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    uvm_reg_hw_reset_seq rs = uvm_reg_hw_reset_seq::type_id::create("rs");
    phase.raise_objection(this);
    rs.model = blk;
    rs.start(null);
    $display("T|reset_seq|done");
    phase.drop_objection(this);
  endtask
endclass

class ral_mismatch_test extends ral_base;
  `uvm_component_utils(ral_mismatch_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    uvm_status_e st;
    phase.raise_objection(this);
    // change CTRL behind the model's back
    @(negedge vif.clk); vif.hw_ctrl = 32'h11; vif.hw_ctrl_we = 1;
    @(negedge vif.clk); vif.hw_ctrl_we = 0;
    blk.CTRL.mirror(st, UVM_CHECK);   // must report the mismatch ...
    $display("T|mismatch|mirror=%0h", mv(blk.CTRL));
    blk.CTRL.mirror(st, UVM_CHECK);   // ... and agree once resynchronised
    $display("T|mismatch|done");
    phase.drop_objection(this);
  endtask
endclass

class ral_pred_test extends ral_base;
  `uvm_component_utils(ral_pred_test)
  function new(string name, uvm_component parent); super.new(name, parent); explicit_predict = 1; endfunction
  task run_phase(uvm_phase phase);
    uvm_status_e st; uvm_reg_data_t d;
    braw_seq raw = braw_seq::type_id::create("raw");
    phase.raise_objection(this);
    // a write the register model never issued still reaches the mirror
    raw.addr = 'h10; raw.data = 32'h1234;
    raw.start(sqr);
    #1;
    $display("T|pred|raw scratch mirror=%0h", mv(blk.SCRATCH));
    // a model-issued write: with auto-predict off, the predictor updates it
    blk.CTRL.write(st, 32'h77);
    #1;
    $display("T|pred|ctrl mirror=%0h", mv(blk.CTRL));
    // a read of an RC register: the predictor applies read-clear
    hw(0, 32'h09);
    blk.EVENTS.read(st, d);
    #1;
    $display("T|pred|events read=%0h mirror=%0h", d, mv(blk.EVENTS));
    phase.drop_objection(this);
  endtask
endclass

class ral_mem_test extends ral_base;
  `uvm_component_utils(ral_mem_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    uvm_status_e st; uvm_reg_data_t d;
    phase.raise_objection(this);
    for (int i = 0; i < 8; i += 3) blk.MEM.write(st, i, 32'hA000 + i);
    for (int i = 0; i < 8; i++) begin
      blk.MEM.read(st, i, d);
      $display("T|mem|%0d=%0h", i, d);
    end
    $display("T|mem|addr3=%0h size=%0d", blk.MEM.get_address(3), blk.MEM.get_size());
    phase.drop_objection(this);
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  rbus_if bif(clk);
  rdut dut(bif);
  initial begin
    uvm_config_db#(virtual rbus_if)::set(null, "uvm_test_top", "vif", bif);
    run_test();
  end
endmodule
