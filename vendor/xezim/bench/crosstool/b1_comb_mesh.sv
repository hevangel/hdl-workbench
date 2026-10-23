// B1 — combinational propagation depth.
//
// A chain of STAGES purely combinational stages fed by a clocked LFSR. Every
// clock edge invalidates stage[0], and the change has to ripple through all
// STAGES continuous assignments before the next edge samples the tail. This
// isolates the settle/propagation engine: event scheduling and evaluation of
// continuous assignments, with almost no procedural code.
//
// Deterministic by construction (no $random, no randomize) so CHECKSUM must be
// identical on every conforming simulator.
//
// Size: -DBENCH_SMALL / default / -DBENCH_LARGE (see README).
`timescale 1ns/1ps

module bench_comb;

`ifdef BENCH_SMALL
  localparam int STAGES = 16;
  localparam int CYCLES = 200;
`elsif BENCH_LARGE
  localparam int STAGES = 128;
  localparam int CYCLES = 20000;
`else
  localparam int STAGES = 64;
  localparam int CYCLES = 4000;
`endif
  localparam int W = 32;   // fixed: the mixing function below assumes 32 bits

  logic             clk = 1'b0;
  logic [W-1:0]     seed = 32'h1234_5678;
  logic [W-1:0]     stage [0:STAGES];
  logic [63:0]      csum = 64'd0;
  int               cyc  = 0;
  bit               failed = 1'b0;

  always #5 clk = ~clk;

  assign stage[0] = seed;

  // One combinational stage per generate iteration: rotate-left, xor a shifted
  // copy, xor the stage index. Cheap per stage, so wall time tracks the number
  // of propagation events rather than arithmetic cost.
  genvar g;
  generate
    for (g = 0; g < STAGES; g++) begin : mesh
      assign stage[g+1] = {stage[g][W-2:0], stage[g][W-1]} ^ (stage[g] >> 3) ^ g;
    end
  endgenerate

  always @(posedge clk) begin
    // Maximal-length-style LFSR (taps 32,22,2,1) — same sequence everywhere.
    seed <= {seed[W-2:0], seed[W-1] ^ seed[21] ^ seed[1] ^ seed[0]};
    csum <= csum + {32'd0, stage[STAGES]};
    cyc  <= cyc + 1;
  end

  // Structural self-check: the first stage must always equal its declared
  // function of stage[0]. Catches a settle engine that samples a stale value.
  always @(posedge clk) begin
    if (stage[1] !== ({stage[0][W-2:0], stage[0][W-1]} ^ (stage[0] >> 3))) begin
      failed <= 1'b1;
    end
  end

  initial begin
    wait (cyc == CYCLES);
    if (stage[STAGES] === 'x || csum === 'x) failed = 1'b1;
    $display("BENCH b1_comb_mesh %s", failed ? "FAIL" : "PASS");
    $display("CHECKSUM b1_comb_mesh %h", csum);
    // Propagation events performed: one per stage per cycle.
    $display("WORK b1_comb_mesh %0d stage_evals", CYCLES * STAGES);
    $finish;
  end

endmodule
