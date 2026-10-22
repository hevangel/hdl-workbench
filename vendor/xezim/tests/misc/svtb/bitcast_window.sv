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

package flat_pkg;
   typedef struct packed {
      logic [1:0] [63:0] wdata;
      logic [1:0] [7:0]  mask;
      logic [1:0]        amask;
   } lane_grp_t;
   typedef struct packed {
      lane_grp_t [1:0] wdata;
   } lane_grp_pair_t;
endpackage
module tb_bitcast;
   import flat_pkg::*;
   `SVTEST_INIT
   logic clk;
   logic reset;
   flat_pkg::lane_grp_t [0:0] [1:0] grp_arr;
   flat_pkg::lane_grp_pair_t              drv_struct;
  typedef logic[$bits(grp_arr)-1:0] flat_cast_t ;
  flat_cast_t flat_view ;
   initial begin
      clk = 0;
      forever #5 clk = ~clk;
   end
   initial begin
      reset = 1'b1;
      grp_arr = '0;
      drv_struct = '0;
      repeat(3) @(posedge clk);
      @(negedge clk);
      reset = 1'b0;
      @(negedge clk);
      drv_struct.wdata[0].wdata[0] = 64'h0000_0000_0000_022e;
      drv_struct.wdata[0].wdata[1] = 64'h0000_4000_0000_0000;
      drv_struct.wdata[0].mask     = '{8'h8b, 8'hc0};
      drv_struct.wdata[0].amask    = 2'b01;
      drv_struct.wdata[1].wdata[0] = 64'h0000_0000_0000_023e;
      drv_struct.wdata[1].wdata[1] = 64'h0000_0000_0000_023f;
      drv_struct.wdata[1].mask     = '{8'h00, 8'h00};
      drv_struct.wdata[1].amask    = 2'b00;
      grp_arr = drv_struct;
      repeat(2) @(posedge clk);
      #1;
      $display("--- MWE RUNTIME PROFILE ---");
     flat_view = flat_cast_t'(grp_arr);
      $display("PROBE flat_tb  = %h", grp_arr);
      $display("PROBE bits tb=%0d elem=%0d memb=%0d", $bits(grp_arr), $bits(grp_arr[0][0]), $bits(grp_arr[0][0].wdata[0]));
     $display("PROBE by_hand  = %h", flat_view[81:18]);
     `SVTEST_CHECK(flat_view[81:18] !== 'x, "RUNTIME BUG: Unpacked variable bit-range part-select returned corrupted X states!")
      repeat(2) @(posedge clk);
      `SVTEST_PASSFAIL
      $finish;
   end
endmodule
