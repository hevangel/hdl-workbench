// B4 — class-based testbench runtime: objects, queues, dynamic arrays,
// associative arrays, strings, mailbox and fork/join.
//
// This is the axis that separates a "simulator" from an "RTL simulator": UVM-
// style testbenches spend most of their time here, not in the design. The
// benchmark allocates and frees many objects, moves them through a mailbox
// between two processes, sorts a queue, and does string formatting and
// associative-array lookups.
//
// Deliberately avoids `randomize()` — constraint solving is not comparable
// across tools (different solvers pick different legal values). Data comes from
// an explicit LFSR so CHECKSUM must match everywhere.
`timescale 1ns/1ps

module bench_oop;

`ifdef BENCH_SMALL
  localparam int N_PKT = 2000;
`elsif BENCH_LARGE
  localparam int N_PKT = 200000;
`else
  localparam int N_PKT = 40000;
`endif
  localparam int PAYLOAD = 8;

  function automatic logic [31:0] lfsr32(input logic [31:0] s);
    lfsr32 = {s[30:0], s[31] ^ s[21] ^ s[1] ^ s[0]};
  endfunction

  class packet;
    int          id;
    logic [31:0] payload [];
    string       tag;

    function new(int id_, logic [31:0] seed_);
      id = id_;
      payload = new[PAYLOAD];
      payload[0] = seed_;
      for (int i = 1; i < PAYLOAD; i++) payload[i] = lfsr32(payload[i-1]);
      tag = $sformatf("pkt_%0d", id_ % 97);
    endfunction

    function logic [31:0] fold();
      logic [31:0] acc = 32'd0;
      foreach (payload[i]) acc = acc ^ payload[i];
      return acc;
    endfunction
  endclass

  mailbox #(packet) mbx = new(64);
  int          by_tag [string];
  logic [63:0] csum = 64'd0;
  int          produced = 0, consumed = 0;
  bit          failed = 1'b0;

  initial begin
    packet q[$];

    fork
      // Producer
      begin
        automatic logic [31:0] seed = 32'hCAFE_0001;
        for (int i = 0; i < N_PKT; i++) begin
          // §6.21: a declaration inside a loop body is implicitly STATIC, and a
          // static declaration may not have a non-static initializer — so this
          // must be explicitly automatic to be legal (and to get a fresh object
          // per iteration).
          automatic packet p = new(i, seed);
          seed = lfsr32(seed);
          mbx.put(p);
          produced++;
        end
      end
      // Consumer: folds each packet, counts by tag, keeps a rolling queue.
      begin
        for (int i = 0; i < N_PKT; i++) begin
          automatic packet p;
          mbx.get(p);
          csum = csum + {32'd0, p.fold()};
          if (by_tag.exists(p.tag)) by_tag[p.tag]++;
          else                      by_tag[p.tag] = 1;
          q.push_back(p);
          if (q.size() > 256) void'(q.pop_front());
          consumed++;
        end
      end
    join

    // --- self-checks -------------------------------------------------
    if (produced != N_PKT || consumed != N_PKT) begin
      $display("FAIL b4: produced %0d consumed %0d, expected %0d", produced, consumed, N_PKT);
      failed = 1'b1;
    end
    if (q.size() != ((N_PKT > 256) ? 256 : N_PKT)) begin
      $display("FAIL b4: queue size %0d unexpected", q.size());
      failed = 1'b1;
    end
    begin
      automatic int total = 0;
      foreach (by_tag[t]) total += by_tag[t];
      if (total != N_PKT) begin
        $display("FAIL b4: associative-array counts total %0d, expected %0d", total, N_PKT);
        failed = 1'b1;
      end
      if (by_tag.num() != ((N_PKT > 97) ? 97 : N_PKT)) begin
        $display("FAIL b4: %0d distinct tags, expected %0d",
                 by_tag.num(), (N_PKT > 97) ? 97 : N_PKT);
        failed = 1'b1;
      end
    end
    // Sorting a queue of handles by a class member exercises the with-clause.
    q.sort with (item.id);
    if (q.size() > 1 && q[0].id > q[q.size()-1].id) begin
      $display("FAIL b4: queue sort did not order by id");
      failed = 1'b1;
    end

    $display("BENCH b4_oop_tb %s", failed ? "FAIL" : "PASS");
    $display("CHECKSUM b4_oop_tb %h", csum);
    $display("WORK b4_oop_tb %0d packet_ops", N_PKT * (PAYLOAD + 4));
    $finish;
  end

endmodule
