package test_tracker_pkg;
  int failures = 0;
  bit test_status[100];
endpackage
`define SVTEST_INIT4 import test_tracker_pkg::*;
`define SVTEST_CHECK_TRACK(index, expr, msg) \
  if (!(expr)) begin \
    test_tracker_pkg::failures++; \
    test_tracker_pkg::test_status[index] = 1'b0; \
    $display("FAIL: %s at time %0t ps", msg, $time); \
  end \
  else begin \
    test_tracker_pkg::test_status[index] = 1'b1; \
  end
`define SVTEST_PASSFAIL4 \
  if (test_tracker_pkg::failures == 0) begin \
    $display("TEST_PASS"); \
  end else begin \
    $display("TEST_FAIL count=%0d", test_tracker_pkg::failures); \
    $fatal(1); \
  end
typedef enum int {
  CLIENT_A,
  CLIENT_B
} ClientID;
typedef enum int {
  DBG_CLIENT_0,
  DBG_CLIENT_1
} DebugClientID;
interface chan_if #(parameter NUM_CHANNELS = 4);
  logic [NUM_CHANNELS-1:0] req;
  modport driver (output req);
  modport receiver (input req);
endinterface
interface debug_if;
  logic debug_enable;
  modport driver (output debug_enable);
  modport receiver (input debug_enable);
endinterface
interface sync_counter_if
#(
  parameter EVENT_COUNTER_WIDTH = 8,
  parameter NUMBER_OF_SYNC_COUNTERS = 2
);
  logic [EVENT_COUNTER_WIDTH-1:0] counter[NUMBER_OF_SYNC_COUNTERS];
  modport receiver (input counter);
endinterface
module simple_dut
(
  chan_if.receiver        req,
  debug_if.receiver      dbg,
  sync_counter_if        sync
);
  always @(*) begin
    if (dbg.debug_enable)
      $display("[%0t] DUT sees request = %b", $time, req.req);
  end
endmodule
class vif_manager;
  virtual chan_if #(4).driver req_vif;
  virtual debug_if.driver    dbg_vif;
  virtual sync_counter_if #(8,2).receiver sync_vif;
  function new(
      virtual chan_if #(4).driver req_vif,
      virtual debug_if.driver dbg_vif,
      virtual sync_counter_if #(8,2).receiver sync_vif
  );
    this.req_vif  = req_vif;
    this.dbg_vif  = dbg_vif;
    this.sync_vif = sync_vif;
  endfunction
  function automatic virtual chan_if #(4).driver
      get_req_interface(input ClientID client, input bit is_proxy);
      return req_vif;
  endfunction
  function automatic virtual debug_if.driver
      get_debug_interface(input DebugClientID debug_client_id);
      return dbg_vif;
  endfunction
  function automatic virtual sync_counter_if #(8,2).receiver
      get_sync_counter_interface();
      return sync_vif;
  endfunction
endclass
module tb_vif_mgr;
  `SVTEST_INIT4
  chan_if #(4)           req_bus();
  debug_if              dbg_bus();
  sync_counter_if #(8,2) sync_bus();
  simple_dut dut (
      .req(req_bus),
      .dbg(dbg_bus),
      .sync(sync_bus)
  );
  vif_manager mgr;
  virtual chan_if #(4).driver req_drv;
  virtual debug_if.driver dbg_drv;
  virtual sync_counter_if #(8,2).receiver sync_rcv;
  initial begin
    mgr = new(req_bus.driver, dbg_bus.driver, sync_bus.receiver);
    req_drv  = mgr.get_req_interface(CLIENT_A, 0);
    dbg_drv  = mgr.get_debug_interface(DBG_CLIENT_0);
    sync_rcv = mgr.get_sync_counter_interface();
    dbg_drv.debug_enable = 1'b1;
    req_drv.req          = 4'b1010;
    sync_bus.counter[0]  = 8'd21;
    sync_bus.counter[1]  = 8'd99;
    #1;
    `SVTEST_CHECK_TRACK(0, req_drv.req == 4'b1010, "Request value mismatch")
    `SVTEST_CHECK_TRACK(1, dbg_drv.debug_enable == 1'b1, "Debug enable mismatch")
    `SVTEST_CHECK_TRACK(2, sync_rcv.counter[0] == 8'd21, "Counter0 mismatch")
    `SVTEST_CHECK_TRACK(3, sync_rcv.counter[1] == 8'd99, "Counter1 mismatch")
    `SVTEST_PASSFAIL4
    $finish;
  end
endmodule
