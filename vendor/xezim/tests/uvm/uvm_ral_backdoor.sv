import uvm_pkg::*;
`include "uvm_macros.svh"

// ---------------- DUT: 4-register file on a trivial bus ----------------
interface bus_if(input logic clk);
  logic        req;
  logic        wr;
  logic [7:0]  addr;
  logic [31:0] wdata;
  logic [31:0] rdata;
  logic        ack;
endinterface

module regfile(bus_if bif);
  logic [31:0] CTRL;   // offset 0, RW, reset 32'h0000_00C0
  logic [31:0] SCRATCH;// offset 4, RW, reset 0
  initial begin CTRL = 32'h0000_00C0; SCRATCH = '0; bif.ack = 0; bif.rdata = '0; end
  always @(posedge bif.clk) begin
    bif.ack <= 1'b0;
    if (bif.req) begin
      if (bif.wr) begin
        case (bif.addr)
          8'h0: CTRL    <= bif.wdata;
          8'h4: SCRATCH <= bif.wdata;
        endcase
      end else begin
        case (bif.addr)
          8'h0: bif.rdata <= CTRL;
          8'h4: bif.rdata <= SCRATCH;
          default: bif.rdata <= 32'hDEAD_BEEF;
        endcase
      end
      bif.ack <= 1'b1;
    end
  end
endmodule

// ---------------- UVM: bus transaction + driver ----------------
class bus_txn extends uvm_sequence_item;
  rand bit        write;
  rand bit [7:0]  addr;
  rand bit [31:0] data;
  `uvm_object_utils_begin(bus_txn)
    `uvm_field_int(write, UVM_ALL_ON)
    `uvm_field_int(addr, UVM_ALL_ON)
    `uvm_field_int(data, UVM_ALL_ON)
  `uvm_object_utils_end
  function new(string name = "bus_txn"); super.new(name); endfunction
endclass

class bus_driver extends uvm_driver #(bus_txn);
  `uvm_component_utils(bus_driver)
  virtual bus_if vif;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    if (!uvm_config_db#(virtual bus_if)::get(this, "", "vif", vif))
      `uvm_fatal("NOVIF", "bus_if not set")
  endfunction
  task run_phase(uvm_phase phase);
    bus_txn tr;
    vif.req <= 0;
    forever begin
      seq_item_port.get_next_item(tr);
      @(posedge vif.clk);
      vif.req   <= 1'b1;
      vif.wr    <= tr.write;
      vif.addr  <= tr.addr;
      vif.wdata <= tr.data;
      @(posedge vif.clk);
      vif.req <= 1'b0;
      @(posedge vif.clk); // ack/rdata settle
      if (!tr.write) tr.data = vif.rdata;
      seq_item_port.item_done();
    end
  endtask
endclass

// ---------------- RAL: registers, block, adapter ----------------
class ctrl_reg_t extends uvm_reg;
  `uvm_object_utils(ctrl_reg_t)
  rand uvm_reg_field EN;
  rand uvm_reg_field MODE;
  function new(string name = "ctrl_reg_t");
    super.new(name, 32, UVM_NO_COVERAGE);
  endfunction
  virtual function void build();
    EN   = uvm_reg_field::type_id::create("EN");
    MODE = uvm_reg_field::type_id::create("MODE");
    // reset value 32'hC0: EN bit6=1, MODE bits[7:7]=1 -> pick simple split:
    // EN = bits[7:0] reset 8'hC0, MODE = bits[15:8] reset 0
    EN.configure(this, 8, 0, "RW", 0, 8'hC0, 1, 1, 0);
    MODE.configure(this, 8, 8, "RW", 0, 8'h00, 1, 1, 0);
  endfunction
endclass

class scratch_reg_t extends uvm_reg;
  `uvm_object_utils(scratch_reg_t)
  rand uvm_reg_field VAL;
  function new(string name = "scratch_reg_t");
    super.new(name, 32, UVM_NO_COVERAGE);
  endfunction
  virtual function void build();
    VAL = uvm_reg_field::type_id::create("VAL");
    VAL.configure(this, 32, 0, "RW", 0, 32'h0, 1, 1, 0);
  endfunction
endclass

class regmodel_t extends uvm_reg_block;
  `uvm_object_utils(regmodel_t)
  rand ctrl_reg_t    CTRL;
  rand scratch_reg_t SCRATCH;
  function new(string name = "regmodel_t");
    super.new(name, UVM_NO_COVERAGE);
  endfunction
  virtual function void build();
    CTRL = ctrl_reg_t::type_id::create("CTRL");
    CTRL.configure(this, null, "CTRL");
    CTRL.build();
    SCRATCH = scratch_reg_t::type_id::create("SCRATCH");
    SCRATCH.configure(this, null, "SCRATCH");
    SCRATCH.build();
    add_hdl_path("top.dut");
    default_map = create_map("default_map", 'h0, 4, UVM_LITTLE_ENDIAN);
    default_map.add_reg(CTRL,    'h0, "RW");
    default_map.add_reg(SCRATCH, 'h4, "RW");
  endfunction
endclass

class bus_adapter extends uvm_reg_adapter;
  `uvm_object_utils(bus_adapter)
  function new(string name = "bus_adapter"); super.new(name); endfunction
  virtual function uvm_sequence_item reg2bus(const ref uvm_reg_bus_op rw);
    bus_txn tr = bus_txn::type_id::create("tr");
    tr.write = (rw.kind == UVM_WRITE);
    tr.addr  = rw.addr[7:0];
    tr.data  = rw.data;
    return tr;
  endfunction
  virtual function void bus2reg(uvm_sequence_item bus_item, ref uvm_reg_bus_op rw);
    bus_txn tr;
    if (!$cast(tr, bus_item)) `uvm_fatal("CAST", "not a bus_txn")
    rw.kind = tr.write ? UVM_WRITE : UVM_READ;
    rw.addr = tr.addr;
    rw.data = tr.data;
    rw.status = UVM_IS_OK;
  endfunction
endclass

// ---------------- env + test ----------------
class ral_env extends uvm_env;
  `uvm_component_utils(ral_env)
  bus_driver                 drv;
  uvm_sequencer #(bus_txn)   sqr;
  regmodel_t                 model;
  bus_adapter                adapter;
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  function void build_phase(uvm_phase phase);
    drv = bus_driver::type_id::create("drv", this);
    sqr = uvm_sequencer#(bus_txn)::type_id::create("sqr", this);
    model = regmodel_t::type_id::create("model");
    model.build();
    model.lock_model();
    adapter = bus_adapter::type_id::create("adapter");
  endfunction
  function void connect_phase(uvm_phase phase);
    drv.seq_item_port.connect(sqr.seq_item_export);
    model.default_map.set_sequencer(sqr, adapter);
    model.default_map.set_auto_predict(1);
  endfunction
endclass

class ral_test extends uvm_test;
  `uvm_component_utils(ral_test)
  ral_env env;
  int failures = 0;
  function new(string name = "ral_test", uvm_component parent = null);
    super.new(name, parent);
  endfunction
  function void build_phase(uvm_phase phase);
    env = ral_env::type_id::create("env", this);
  endfunction
  task chk(bit ok, string what);
    if (!ok) begin failures++; $display("FAIL: %s", what); end
    else $display("PASS: %s", what);
  endtask
  task run_phase(uvm_phase phase);
    uvm_status_e status;
    uvm_reg_data_t data;
    phase.raise_objection(this);
    #20;
    env.model.reset();
    // ---- BACKDOOR-only probes (no sequencer involved) ----
    env.model.CTRL.read(status, data, UVM_BACKDOOR);
    chk(status == UVM_IS_OK, "CTRL backdoor read status");
    chk(data == 32'h0000_00C0, $sformatf("CTRL backdoor read reset (got %h)", data));
    env.model.SCRATCH.write(status, 32'hFACE_CAFE, UVM_BACKDOOR);
    chk(status == UVM_IS_OK, "SCRATCH backdoor write status");
    env.model.SCRATCH.read(status, data, UVM_BACKDOOR);
    chk(data == 32'hFACE_CAFE, $sformatf("SCRATCH backdoor read-back (got %h)", data));
    if (failures == 0) $display("TEST_PASS");
    else $display("TEST_FAIL count=%0d", failures);
    phase.drop_objection(this);
  endtask
  task never_used(uvm_phase phase);
    uvm_status_e status;
    uvm_reg_data_t data;
    #20;
    env.model.reset();
    // 1. reset/mirror values
    chk(env.model.CTRL.get_reset() == 32'h0000_00C0, "CTRL reset value = C0");
    chk(env.model.CTRL.get_mirrored_value() == 32'h0000_00C0, "CTRL mirror starts at reset");
    // 2. frontdoor READ of reset value from DUT
    env.model.CTRL.read(status, data, UVM_FRONTDOOR);
    chk(status == UVM_IS_OK, "CTRL frontdoor read status");
    chk(data == 32'h0000_00C0, $sformatf("CTRL frontdoor read reset value (got %h)", data));
    // 3. frontdoor WRITE + mirror update (auto predict)
    env.model.SCRATCH.write(status, 32'hA5A5_5A5A, UVM_FRONTDOOR);
    chk(status == UVM_IS_OK, "SCRATCH frontdoor write status");
    chk(env.model.SCRATCH.get_mirrored_value() == 32'hA5A5_5A5A, "SCRATCH mirror after write");
    // 4. frontdoor read-back
    env.model.SCRATCH.read(status, data, UVM_FRONTDOOR);
    chk(data == 32'hA5A5_5A5A, $sformatf("SCRATCH read-back (got %h)", data));
    // 5. field ops: write MODE field alone (read-modify-write)
    env.model.CTRL.MODE.write(status, 8'h3A);
    env.model.CTRL.read(status, data, UVM_FRONTDOOR);
    chk(data == 32'h0000_3AC0, $sformatf("CTRL after MODE field write (got %h)", data));
    // 6. desired/mirror API: set + update
    env.model.SCRATCH.set(32'h1234_5678);
    env.model.SCRATCH.update(status, UVM_FRONTDOOR);
    env.model.SCRATCH.read(status, data, UVM_FRONTDOOR);
    chk(data == 32'h1234_5678, $sformatf("SCRATCH after set+update (got %h)", data));
    // 7. mirror() with check against DUT
    env.model.SCRATCH.mirror(status, UVM_CHECK, UVM_FRONTDOOR);
    chk(status == UVM_IS_OK, "SCRATCH mirror(UVM_CHECK)");
    if (failures == 0) $display("TEST_PASS");
    else $display("TEST_FAIL count=%0d", failures);
    phase.drop_objection(this);
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bif(clk);
  regfile dut(bif);
  initial begin
    uvm_config_db#(virtual bus_if)::set(null, "*", "vif", bif);
    run_test("ral_test");
  end
endmodule
