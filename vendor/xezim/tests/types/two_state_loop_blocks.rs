//! Clocked blocks whose loop counter is an `int` register now run on the
//! two-state executor. Three things had to hold for that:
//!
//! * `SetSigned` (an `int` counter), `MoveResize` (its increment) and the
//!   dynamic-offset slice read / range stores that loop bodies address
//!   memories with all lower;
//! * a register's KNOWN VALUE is forgotten at a loop head — `LoadConst(i, 0)`
//!   seeded `rc[i] = 0`, and an element read indexed by `i` inside the loop
//!   lowered as a constant read of element 0 on every trip (the block used
//!   to bail on `SetSigned` before it got that far);
//! * the bytecode diet that precedes lowering (copy forwarding into readers,
//!   `And`/`Mul`/`Sub`/`Or` constant operands, `Resize;Move` folding)
//!   preserves values.
//!
//! Expected values come from the four-state interpreter path these blocks
//! took before, which is reference-verified for these shapes.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// `q[p] <= mem[aa[p]]` with `aa[p]` itself advanced by `p + 1`: every
/// element of `q` depends on its OWN lane's pointer, so a counter frozen at
/// its seed shows up as lanes 3 and 7 never written.
#[test]
fn int_counter_indexes_arrays_per_lane() {
    let msgs = messages(
        "module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [31:0] mem [0:255];
  logic [7:0] aa [0:7];
  logic [31:0] q [0:7];
  int cyc = 0;
  always @(posedge clk) begin
    cyc <= cyc + 1;
    for (int p = 0; p < 8; p++) begin
      q[p]  <= mem[aa[p]];
      aa[p] <= aa[p] + 8'(p + 1);
    end
  end
  initial begin
    for (int k = 0; k < 256; k++) mem[k] = 32'(k * 32'h01010101 + 7);
    foreach (aa[p]) aa[p] = 8'(p * 31);
    foreach (q[p]) q[p] = '0;
    repeat (40) @(posedge clk);
    #1 $display(\"SRAM %h %h %h %0d\", q[0], q[3], q[7], cyc);
    $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("SRAM 2727272e f9f9fa00 11111118 40")),
        "per-lane pointer loop: {msgs:?}"
    );
}

/// A packed 2-D queue written through a per-lane write pointer read from an
/// unpacked array: dynamic range store into a packed vector, element load
/// and element NBA store in one loop.
#[test]
fn packed_queue_through_element_pointer() {
    let msgs = messages(
        "module tb;
  logic clk = 0; always #5 clk = ~clk;
  typedef struct packed { logic [7:0] a; } e_t;
  e_t [3:0][7:0] bqueue;
  logic [2:0] wptr [0:3];
  always @(posedge clk)
    for (int i = 0; i < 4; i++) begin
      bqueue[i][wptr[i]] <= e_t'(8'(i * 8'h11 + 1));
      wptr[i] <= wptr[i] + 1;
    end
  initial begin
    bqueue = '0; foreach (wptr[i]) wptr[i] = 3'(i);
    repeat (3) @(posedge clk);
    #1 $display(\"QUEUE %h\", bqueue);
    $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter().any(|m| m
            .contains("QUEUE 0000343434000000000000232323000000000000121212000000000000010101")),
        "packed queue: {msgs:?}"
    );
}

/// The lane-table and byte-lane memory shapes: a chained dynamic packed read
/// (`sbox_i[(b+m)&15][m]`) sliced from the signal in place, a dynamic
/// blocking range store, and `mem[ab][(i*8) +: 8] <=` non-blocking stores.
#[test]
fn lane_table_and_byte_lane_memory() {
    let msgs = messages(
        "module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [15:0][7:0] sbox_o, sbox_i;
  logic [31:0][31:0] mem;
  logic [31:0] db; logic [4:0] ab; int cyc = 0;
  always @(posedge clk) begin
    for (int b = 0; b < 16; b++)
      for (int m = 0; m < 8; m++)
        sbox_o[b][m] = sbox_i[(b + m) & 15][m];
    for (int i = 0; i < 4; i++)
      mem[ab][(i * 8) +: 8] <= db[(i * 8) +: 8];
    cyc <= cyc + 1; ab <= ab + 1; db <= db + 32'h0101_0101;
  end
  initial begin
    sbox_o = '0; mem = '0; db = 32'hdead_beef; ab = 0;
    for (int i = 0; i < 16; i++) sbox_i[i] = 8'(i * 7 + 1);
    repeat (20) @(posedge clk);
    #1 $display(\"LANES %h %h %h %0d\", sbox_o, mem[7], mem[19], cyc);
    $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("LANES 180b0209042f765d40434a517c272e35 e5b4c5f6 f1c0d202 20")),
        "lane table / byte-lane memory: {msgs:?}"
    );
}
