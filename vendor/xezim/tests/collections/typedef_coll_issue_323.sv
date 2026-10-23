
package regs_pkg;
  typedef int unsigned addr_t;
  typedef addr_t addr_array_t[$];
  typedef struct {
    addr_array_t addr_list;
    bit [6:0]    access_attr;
  } reg_map_t [string];                       // struct typedef WITH an AA dimension

  reg_map_t ipw04_regs = '{
    "REG_IPW61_00"     : '{ addr_list: { 'h1000 },                               access_attr: 7'h01 },
    "REG_IPW61_01" : '{ addr_list: { 'h2000, 'h2001, 'h2002, 'h2003, 'h2004 }, access_attr: 7'h02 },
    "REG_IPW61_02" : '{ addr_list: { 'h3000, 'h3001, 'h3002 },        access_attr: 7'h03 },
    "REG_IPW61_03_THRESHOLDS" : '{ addr_list: { 'h4000, 'h4001, 'h4002, 'h4003, 'h4004, 'h4005, 'h4006, 'h4007,
                                                       'h4008, 'h4009, 'h400a, 'h400b, 'h400c, 'h400d, 'h400e, 'h400f,
                                                       'h4010 },                     access_attr: 7'h04 }
  };

  addr_array_t ipw04_init_config_set = {
    ipw04_regs["REG_IPW61_00"].addr_list,
    ipw04_regs["REG_IPW61_01"].addr_list,
    ipw04_regs["REG_IPW61_02"].addr_list,
    ipw04_regs["REG_IPW61_03_THRESHOLDS"].addr_list
  };
endpackage

module tb;
  import regs_pkg::*;
  int errors = 0;
  initial begin
    if (ipw04_regs["REG_IPW61_00"].addr_list.size() != 1) begin
      $display("FAIL: 00 addr_list.size()=%0d (exp 1)", ipw04_regs["REG_IPW61_00"].addr_list.size()); errors++;
    end
    if (ipw04_regs["REG_IPW61_01"].addr_list.size() != 5) begin
      $display("FAIL: 01 addr_list.size()=%0d (exp 5)", ipw04_regs["REG_IPW61_01"].addr_list.size()); errors++;
    end
    if (ipw04_regs["REG_IPW61_02"].addr_list.size() != 3) begin
      $display("FAIL: 02 addr_list.size()=%0d (exp 3)", ipw04_regs["REG_IPW61_02"].addr_list.size()); errors++;
    end
    if (ipw04_regs["REG_IPW61_03_THRESHOLDS"].addr_list.size() != 17) begin
      $display("FAIL: 03 addr_list.size()=%0d (exp 17)", ipw04_regs["REG_IPW61_03_THRESHOLDS"].addr_list.size()); errors++;
    end
    if (ipw04_init_config_set.size() != 26) begin
      $display("FAIL: ipw04_init_config_set.size()=%0d (exp 26)", ipw04_init_config_set.size()); errors++;
    end else begin
      $display("OK: ipw04_init_config_set = %p", ipw04_init_config_set);
    end
    if (errors == 0) $display("TEST PASSED");
    else             $display("TEST FAILED: %0d errors", errors);
  end
endmodule


