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

package cfgspec_pkg;
  typedef logic [31:0] u32_t;
  typedef struct packed {
    logic [7:0]  REG_A;
    logic [7:0]  REG_B;
    u32_t [4:0]  REG_SEC;
    logic [7:0]  REG_C ;
    logic [7:0]  REG_D;
  } cfgspec_pkg_t;
endpackage
module cfg_leaf (
  input  logic               clk,
  input  logic               reset,
  input  logic               write,
  input  logic [4:0]         addr,
  input  logic [31:0]        wdata,
  output cfgspec_pkg::cfgspec_pkg_t cnfg
);
  cfgspec_pkg::cfgspec_pkg_t cnfg_mcp ;
  always_ff @(posedge clk) begin
    if (reset) begin
      cnfg_mcp.REG_A   <= 8'h00;
      cnfg_mcp.REG_B   <= 8'h00;
      cnfg_mcp.REG_SEC[0] <= 32'h0;
      cnfg_mcp.REG_SEC[1] <= 32'h0;
      cnfg_mcp.REG_SEC[2] <= 32'h0;
      cnfg_mcp.REG_SEC[3] <= 32'h0;
      cnfg_mcp.REG_SEC[4] <= 32'h0;
      cnfg_mcp.REG_C   <= 8'h00;
      cnfg_mcp.REG_D   <= 8'h00;
    end else begin
      if (write) begin
        case (addr)
          5'd8:  cnfg_mcp.REG_SEC[0] <= wdata;
          5'd12: cnfg_mcp.REG_SEC[1] <= wdata;
          5'd16: cnfg_mcp.REG_SEC[2] <= wdata;
          5'd20: cnfg_mcp.REG_SEC[3] <= wdata;
          5'd24: cnfg_mcp.REG_SEC[4] <= wdata;
          default: ;
        endcase
      end
    end
  end
  always_comb
    cnfg = cnfg_mcp ;
endmodule: cfg_leaf
module cfg_wrap (
  input  logic         clk,
  input  logic         reset,
  input  logic         write,
  input  logic [4:0]   addr,
  input  logic [31:0]  wdata,
  output logic [159:0] creg_sec
);
  logic [159:0] creg_sec_int;
  cfgspec_pkg::cfgspec_pkg_t cnfg;
  always_ff @(posedge clk) begin
    if (reset) begin
      creg_sec_int <= '0;
    end else begin
      creg_sec_int <= cnfg.REG_SEC;
    end
  end
  assign creg_sec = creg_sec_int;
  cfg_leaf u_spec (
    .clk(clk), .reset(reset), .write(write), .addr(addr), .wdata(wdata), .cnfg(cnfg)
  );
endmodule: cfg_wrap
module cfg_dut (
  input  logic         clk,
  input  logic         reset,
  input  logic         write,
  input  logic [4:0]   addr,
  input  logic [31:0]  wdata,
  output logic [159:0] creg_sec
);
  cfg_wrap u_cfg_wrap (
    .clk(clk), .reset(reset), .write(write), .addr(addr), .wdata(wdata), .creg_sec(creg_sec)
  );
endmodule: cfg_dut
module tb_cfg_regs;
  `SVTEST_INIT
  logic         clk;
  logic         reset;
  logic         write;
  logic [4:0]   addr;
  logic [31:0]  wdata;
  logic [159:0] creg_sec;
  bit clk_free_running = 0;
  bit signals_stabilized = 0;
  initial begin
    clk = 1'b0;
    #3;  clk = 1'bx;
    #4;  clk = 1'b1;
    #2;  clk = 1'b0;
    #6;  clk = 1'bx;
    #2;  clk = 1'b0;
    #5;  clk_free_running = 1;
    forever #5 clk = ~clk;
  end
  cfg_dut u_dut (
    .clk(clk), .reset(reset), .write(write), .addr(addr), .wdata(wdata), .creg_sec(creg_sec)
  );
  task automatic write_cfg_reg(input logic [4:0] target_addr, input logic [31:0] payload);
    @(posedge clk);
    #1;
    write = 1'b1;
    addr  = target_addr;
    wdata = payload;
    @(posedge clk);
    #1;
    write = 1'b0;
    addr  = '0;
    wdata = '0;
  endtask
  initial begin
    write = 1'b0;
    addr  = '0;
    wdata = '0;
    reset = 1'bx;
    #7;  reset = 1'b0;
    #4;  reset = 1'b1;
    #3;  reset = 1'bx;
    wait(clk_free_running == 1'b1);
    @(posedge clk);
    reset = 1'b1;
    @(posedge clk);
    @(posedge clk);
    #1;
    signals_stabilized = 1'b1;
    `SVTEST_CHECK((u_dut.u_cfg_wrap.u_spec.cnfg.REG_SEC == 'h0), "RESET_BUG: Structural leaf configurations failed initialization layout mapping!")
    `SVTEST_CHECK((u_dut.u_cfg_wrap.creg_sec_int == 'h0),       "RESET_BUG: Configuration pipeline mapping memory leaked content past reset boundary!")
    `SVTEST_CHECK((creg_sec == 'h0),                          "RESET_BUG: Top wrapper bus output failed clear mapping alignment!")
    repeat (5) @(posedge clk);
    #1; reset = 1'b0;
    @(posedge clk);
    write_cfg_reg(5'd8, 32'hDEAD_BEEF);
    `SVTEST_CHECK((u_dut.u_cfg_wrap.u_spec.cnfg.REG_SEC[0] == 32'hDEAD_BEEF), "WRITE_BUG: Target array block segment 0 failed storage mapping!")
    write_cfg_reg(5'd24, 32'hCAFE_BABE);
    `SVTEST_CHECK((u_dut.u_cfg_wrap.u_spec.cnfg.REG_SEC[4] == 32'hCAFE_BABE), "WRITE_BUG: Target array block segment 4 failed storage mapping!")
    @(posedge clk);
    #1;
    `SVTEST_CHECK((creg_sec[31:0] == 32'hDEAD_BEEF),    "PIPE_BUG: Downstream structural vector chunk [0] missed bus propagation!")
    `SVTEST_CHECK((creg_sec[159:128] == 32'hCAFE_BABE), "PIPE_BUG: Downstream structural vector chunk [4] missed bus propagation!")
    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
