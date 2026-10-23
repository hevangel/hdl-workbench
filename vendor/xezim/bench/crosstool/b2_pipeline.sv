// B2 — synchronous pipeline: nonblocking-assignment throughput.
//
// DEPTH pipeline registers updated every clock edge from one always_ff block,
// plus a second block that consumes the tail. This isolates the edge-detection
// and NBA (nonblocking assignment) machinery: the number of scheduled updates
// per cycle is DEPTH, and each must land in the NBA region so the shift is a
// true pipeline rather than a collapse.
//
// The latency self-check is the important one: a simulator that applies
// nonblocking updates in the active region would shift the whole pipeline in a
// single cycle and fail it.
`timescale 1ns/1ps

module bench_pipe;

`ifdef BENCH_SMALL
  localparam int DEPTH  = 16;
  localparam int CYCLES = 2000;
`elsif BENCH_LARGE
  localparam int DEPTH  = 256;
  localparam int CYCLES = 50000;
`else
  localparam int DEPTH  = 64;
  localparam int CYCLES = 20000;
`endif
  localparam int W = 32;

  logic         clk = 1'b0;
  logic         rst_n = 1'b0;
  logic [W-1:0] pipe [0:DEPTH-1];
  logic [W-1:0] inval = 32'h89AB_CDEF;
  logic [63:0]  csum = 64'd0;
  int           cyc = 0;
  int           tagged_at = -1;     // cycle the marker entered the pipeline
  int           seen_at = -1;       // cycle it emerged
  bit           failed = 1'b0;

  always #5 clk = ~clk;

  function automatic logic [W-1:0] lfsr32(input logic [W-1:0] s);
    lfsr32 = {s[W-2:0], s[W-1] ^ s[21] ^ s[1] ^ s[0]};
  endfunction

  always_ff @(posedge clk) begin
    if (!rst_n) begin
      for (int i = 0; i < DEPTH; i++) pipe[i] <= '0;
      cyc   <= 0;
      csum  <= 64'd0;
      inval <= 32'h89AB_CDEF;
    end else begin
      inval   <= lfsr32(inval);
      pipe[0] <= inval;
      // Every stage reads its PREDECESSOR's pre-edge value: correct only if
      // the updates are nonblocking.
      for (int i = 1; i < DEPTH; i++) pipe[i] <= pipe[i-1] + i[W-1:0];
      csum <= csum + {32'd0, pipe[DEPTH-1]};
      cyc  <= cyc + 1;
    end
  end

  // Latency check: inject a marker, then confirm it appears at the tail
  // exactly DEPTH cycles later.
  localparam logic [W-1:0] MARKER = 32'hDEAD_BEEF;
  always_ff @(posedge clk) begin
    if (rst_n) begin
      if (pipe[0] == MARKER && tagged_at < 0) tagged_at <= cyc;
      if (pipe[DEPTH-1] == (MARKER + tail_bias()) && tagged_at >= 0 && seen_at < 0) begin
        seen_at <= cyc;
      end
    end
  end

  // Sum of the per-stage `+ i` biases the marker accumulates on its way down.
  function automatic logic [W-1:0] tail_bias();
    logic [W-1:0] acc;
    acc = '0;
    for (int i = 1; i < DEPTH; i++) acc = acc + i[W-1:0];
    tail_bias = acc;
  endfunction

    // Hold reset across at least one clock edge so the reset branch actually
    // executes (releasing it before the first edge leaves the pipe at x).
  initial begin
    repeat (2) @(posedge clk);
    @(negedge clk) rst_n = 1'b1;
    // Inject the marker one cycle after reset release.
    @(posedge clk);
    @(negedge clk);
    force inval = MARKER;
    @(posedge clk);
    release inval;

    wait (cyc >= CYCLES);
    if (seen_at < 0) begin
      $display("FAIL b2: marker never reached the pipeline tail");
      failed = 1'b1;
    end else if ((seen_at - tagged_at) != DEPTH - 1) begin
      $display("FAIL b2: pipeline latency %0d, expected %0d", seen_at - tagged_at, DEPTH - 1);
      failed = 1'b1;
    end
    if (csum === 'x) failed = 1'b1;
    $display("BENCH b2_pipeline %s", failed ? "FAIL" : "PASS");
    $display("CHECKSUM b2_pipeline %h", csum);
    $display("WORK b2_pipeline %0d nba_updates", CYCLES * DEPTH);
    $finish;
  end

endmodule
