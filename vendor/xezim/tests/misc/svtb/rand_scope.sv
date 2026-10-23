`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH
`define SVTEST_INIT \
  int failures = 0;
`define SVTEST_CHECK(expr, msg) \
  if (!(expr)) begin \
    failures++; \
    $display("FAIL @%0t : %s", $time, msg); \
  end
`define SVTEST_PASSFAIL \
  if (failures == 0) begin \
    $display("TEST_PASS"); \
  end else begin \
    $display("TEST_FAIL count=%0d", failures); \
    $fatal(1); \
  end
`endif

module rand_scope_engine;
  bit shared_mode_bit;
  class txn_pkt_c;
    rand bit [7:0] data_payload;
    rand bit [3:0] channel_id;
    constraint c_boundary_integrity {
      if (shared_mode_bit == 1'b1) {
        data_payload inside {[8'h00 : 8'h7F]};
        channel_id   inside {[4'h0  : 4'h7]};
      } else {
        data_payload inside {[8'h80 : 8'hFF]};
        channel_id   inside {[4'h8  : 4'hF]};
      }
    }
  endclass
  initial begin
    `SVTEST_INIT
    txn_pkt_c packet = new();
    shared_mode_bit = 1'b1;
    repeat (50) begin
      `SVTEST_CHECK(packet.randomize(), "RAND_ERROR: Randomization failed in Mode 1")
      `SVTEST_CHECK((packet.data_payload <= 8'h7F), "CONSTRAINT_VIOLATION: data_payload exceeded 0x7F in Mode 1")
      `SVTEST_CHECK((packet.channel_id   <= 4'h7),  "CONSTRAINT_VIOLATION: channel_id exceeded 0x7 in Mode 1")
    end
    shared_mode_bit = 1'b0;
    repeat (50) begin
      `SVTEST_CHECK(packet.randomize(), "RAND_ERROR: Randomization failed in Mode 0")
      `SVTEST_CHECK((packet.data_payload >= 8'h80), "CONSTRAINT_VIOLATION: data_payload fell below 0x80 in Mode 0")
      `SVTEST_CHECK((packet.channel_id   >= 4'h8),  "CONSTRAINT_VIOLATION: channel_id fell below 0x8 in Mode 0")
    end
    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
module tb_rand_scope;
  rand_scope_engine u_rse();
endmodule
