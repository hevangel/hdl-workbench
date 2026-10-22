// B3 — memory/array access: storage representation and indexed access cost.
//
// Fills a large unpacked memory, then performs a long read-modify-write walk
// with LFSR-generated addresses so the access pattern defeats simple locality.
// This is where a simulator's value representation (4-state storage, per-element
// signal objects, dirty tracking) shows up: the arithmetic is trivial, the cost
// is addressing and storing.
//
// Self-check: every location written in the fill phase is read back and
// verified before the random walk begins, so a broken index path fails rather
// than silently producing a different checksum.
`timescale 1ns/1ps

module bench_mem;

`ifdef BENCH_SMALL
  localparam int AW     = 10;      // 1 Ki entries
  localparam int WALKS  = 20000;
`elsif BENCH_LARGE
  localparam int AW     = 16;      // 64 Ki entries
  localparam int WALKS  = 2000000;
`else
  localparam int AW     = 14;      // 16 Ki entries
  localparam int WALKS  = 400000;
`endif
  localparam int DW    = 32;
  localparam int DEPTH = 1 << AW;

  logic [DW-1:0] mem [0:DEPTH-1];
  logic [63:0]   csum = 64'd0;
  bit            failed = 1'b0;

  function automatic logic [31:0] lfsr32(input logic [31:0] s);
    lfsr32 = {s[30:0], s[31] ^ s[21] ^ s[1] ^ s[0]};
  endfunction

  initial begin
    logic [31:0] rnd;
    logic [AW-1:0] addr;
    logic [DW-1:0] rd;

    // Fill phase — deterministic content derived from the index.
    for (int i = 0; i < DEPTH; i++) begin
      mem[i] = (i[DW-1:0] * 32'h9E37_79B9) ^ {16'd0, i[15:0]};
    end

    // Readback verification: catches a broken element-select or storage path.
    for (int i = 0; i < DEPTH; i++) begin
      if (mem[i] !== ((i[DW-1:0] * 32'h9E37_79B9) ^ {16'd0, i[15:0]})) begin
        failed = 1'b1;
        if (i < 4) $display("FAIL b3: mem[%0d] readback mismatch", i);
      end
    end

    // Random-access read-modify-write walk.
    rnd = 32'h0BAD_F00D;
    for (int k = 0; k < WALKS; k++) begin
      rnd  = lfsr32(rnd);
      addr = rnd[AW-1:0];
      rd   = mem[addr];
      mem[addr] = rd + rnd;
      csum = csum + {32'd0, rd};
    end

    if (csum === 'x) failed = 1'b1;
    $display("BENCH b3_memory %s", failed ? "FAIL" : "PASS");
    $display("CHECKSUM b3_memory %h", csum);
    $display("WORK b3_memory %0d accesses", 2 * DEPTH + 2 * WALKS);
    $finish;
  end

endmodule
