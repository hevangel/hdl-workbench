// B5 — elaboration scale: hierarchy flattening, parameter resolution and
// per-instance setup cost.
//
// Unlike B1-B4 this benchmark is dominated by COMPILE/ELABORATE time, not
// simulation: it instantiates CLUSTERS x LEAVES parameterized leaves, each
// carrying its own functions, tasks, localparams and procedural blocks, and
// then runs only a handful of cycles. Time your tool's separate elaborate step
// (for xezim, `--compile`) as well as end to end.
//
// This is the axis where flattening elaborators and reference-counting
// elaborators differ by orders of magnitude, so it is the one most worth
// reporting separately from run time.
`timescale 1ns/1ps

module leafm #(
  parameter int ID = 0,
  parameter int W  = 8
) (
  input  logic         clk,
  input  logic [W-1:0] din,
  output logic [W-1:0] dout
);
  localparam int MIX  = (ID * 7) % 251;
  localparam int MASK = (1 << W) - 1;

  logic [W-1:0] r0, r1;

  function automatic logic [W-1:0] mixf(input logic [W-1:0] a);
    mixf = ((a << 1) ^ (a >> 2) ^ MIX[W-1:0]) & MASK[W-1:0];
  endfunction

  task automatic step(input logic [W-1:0] a);
    r0 <= mixf(a);
    r1 <= r0 ^ a;
  endtask

  always_ff @(posedge clk) step(din);
  assign dout = r0 ^ r1;
endmodule

module cluster #(
  parameter int BASE   = 0,
  parameter int LEAVES = 16,
  parameter int W      = 8
) (
  input  logic         clk,
  input  logic [W-1:0] din,
  output logic [W-1:0] dout
);
  logic [W-1:0] chain [0:LEAVES];
  assign chain[0] = din;
  genvar g;
  generate
    for (g = 0; g < LEAVES; g++) begin : lg
      localparam int LEAF_ID = BASE + g;
      leafm #(.ID(LEAF_ID), .W(W)) u (.clk(clk), .din(chain[g]), .dout(chain[g+1]));
    end
  endgenerate
  assign dout = chain[LEAVES];
endmodule

module bench_elab;

`ifdef BENCH_SMALL
  localparam int CLUSTERS = 4;
  localparam int LEAVES   = 8;
`elsif BENCH_LARGE
  localparam int CLUSTERS = 64;
  localparam int LEAVES   = 32;
`else
  localparam int CLUSTERS = 16;
  localparam int LEAVES   = 16;
`endif
  localparam int W = 8;
  // Each leaf contributes TWO register stages before its output is defined
  // (`r1` depends on the previous `r0`), so the chain needs 2 cycles per leaf
  // to flush. Skip that fill when accumulating, or the x's latch into the
  // checksum forever.
  localparam int FILL   = 2 * CLUSTERS * LEAVES + 8;
  localparam int CYCLES = FILL + 64;

  logic         clk = 1'b0;
  logic [W-1:0] tops [0:CLUSTERS];
  logic [63:0]  csum = 64'd0;
  int           cyc = 0;
  bit           failed = 1'b0;

  always #5 clk = ~clk;

  // Drive the chain with a CHANGING source: a constant input would make every
  // leaf transparent in steady state, so the checksum would not depend on the
  // hierarchy at all (and a mid-chain error would go unnoticed).
  logic [W-1:0] src = 8'hA5;
  always_ff @(posedge clk) src <= {src[W-2:0], src[W-1] ^ src[4] ^ src[3] ^ src[2]};
  assign tops[0] = src;
  genvar c;
  generate
    for (c = 0; c < CLUSTERS; c++) begin : cg
      cluster #(.BASE(c * LEAVES), .LEAVES(LEAVES), .W(W))
        u (.clk(clk), .din(tops[c]), .dout(tops[c+1]));
    end
  endgenerate

  always @(posedge clk) begin
    cyc <= cyc + 1;
    if (cyc >= FILL) csum <= csum + {56'd0, tops[CLUSTERS]};
  end

  initial begin
    wait (cyc == CYCLES);
    // Every leaf must have elaborated: the tail is driven only if the whole
    // chain exists, and a missing instance leaves it x.
    if (tops[CLUSTERS] === 'x || csum === 'x) begin
      $display("FAIL b5: hierarchy tail undriven — instances missing");
      failed = 1'b1;
    end
    $display("BENCH b5_elab_scale %s", failed ? "FAIL" : "PASS");
    $display("CHECKSUM b5_elab_scale %h", csum);
    $display("WORK b5_elab_scale %0d instances", CLUSTERS * LEAVES);
    $finish;
  end

endmodule
