//! PERFORMANCE REGRESSION guard for the loop-heavy clocked-block path —
//! asserts on deterministic WORK COUNTERS, like `work_counters.rs`.
//!
//! The shapes here are the DRAM-model family: `int`-counter loops over lane
//! tables, byte-lane writes into a packed memory, per-lane pointers into a
//! packed queue, constant-bound range stores from generate arms, and a block
//! that keeps reading an x element. Each test asserts the ANSWER first, then
//! the counter that made the shape fast: the block runs on the two-state
//! executor (`two_state_evals`), its bytecode stays lean (`len=`), or the
//! executor stops re-trying a block that always bails on x (`x_read=`).
//! Bounds are CEILINGS; re-baseline when an optimization lowers them.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    run_env(name, src, &[])
}

fn run_env(name: &str, src: &str, env: &[(&str, &str)]) -> String {
    let dir =
        std::env::temp_dir().join(format!("xezim_loop_block_counters_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temporary directory");
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).expect("write temporary design");
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb",
            "--no-cache",
            path.to_str().unwrap(),
        ])
        // These work counters belong to the bytecode executor; native
        // execution bypasses them even when the design's answers match.
        .env("XEZIM_JIT", "0")
        .env("XEZIM_PROFILE_REPORT", "1")
        .env("XEZIM_EDGE_BLOCK_STATS", "1")
        .envs(env.iter().copied())
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "run failed:\n{text}");
    text
}

/// The integer after `key` on the first line containing it.
fn stat(text: &str, key: &str) -> u64 {
    let line = text
        .lines()
        .find(|l| l.contains(key))
        .unwrap_or_else(|| panic!("missing counter `{key}`:\n{text}"));
    let after = &line[line.find(key).unwrap() + key.len()..];
    after
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("unparsable counter `{key}` in `{line}`"))
}

/// Lane table read through a chained dynamic packed select, byte-lane
/// non-blocking stores into a 1024-bit memory, and `int` loop counters:
/// the block must run two-state on every clock, and its bytecode must stay
/// at the dieted length (it was 160 instructions before copy forwarding and
/// the constant-operand kinds; 105 after).
#[test]
fn lane_table_block_is_two_state_and_lean() {
    let text = run(
        "lanes",
        r#"
module tb;
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
    repeat (200) @(posedge clk);
    #1 $display("LANES %h %h %h %0d", sbox_o, mem[7], mem[19], cyc);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("LANES 180b0209042f765d40434a517c272e35 a67586b6 926172a2 200"),
        "answer:\n{text}"
    );
    assert!(
        stat(&text, "two_state_evals=") >= 200,
        "the loop block left two-state:\n{text}"
    );
    let len = stat(&text, "block=0 execs=200 len=");
    assert!(
        len <= 110,
        "the loop block's bytecode grew to {len} instructions:\n{text}"
    );
}

/// Per-lane write pointers read from an unpacked array, indexing a packed
/// queue: element loads, an element non-blocking store and a dynamic range
/// store in one `int` loop.
#[test]
fn queue_pointer_block_is_two_state() {
    let text = run(
        "queue",
        r#"
module tb;
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
    #1 $display("QUEUE %h", bqueue);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("QUEUE 0000343434000000000000232323000000000000121212000000000000010101"),
        "answer:\n{text}"
    );
    assert!(
        stat(&text, "two_state_evals=") >= 3,
        "the queue block left two-state:\n{text}"
    );
}

/// Generate arms whose range bounds are constants only after folding
/// (`v[g*8 +: 8]`, `mem[g][7:4]`): the dynamic-bound and array-element
/// store forms fold to constant stores, which lower. Sixteen c906 and four
/// hundred c910 flops bailed on these two shapes.
#[test]
fn constant_bound_stores_from_generate_arms_are_two_state() {
    // Merging folds the arms into one block; the per-arm decision is what
    // this test is about.
    let text = run_env(
        "genarms",
        r#"
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [31:0] v = 0, din = 32'h0102_0304;
  logic [7:0] mem [0:3];
  logic en = 0; logic [3:0] d = 4'b1010;
  int cyc = 0;
  always @(posedge clk) begin cyc <= cyc + 1; en <= ~en; din <= din + 32'h0101_0101; end
  genvar g;
  generate for (g = 0; g < 4; g = g + 1) begin: L
    always @(posedge clk)
      if (en) v[g*8 +: 8] <= {8{d[g]}};
      else    v[g*8 +: 8] <= din[g*8 +: 8];
    always @(posedge clk)
      mem[g][7:4] <= din[g*8 +: 4];
  end endgenerate
  initial begin
    foreach (mem[k]) mem[k] = 8'h0f;
    repeat (20) @(posedge clk);
    #1 $display("GEN %h %h %h %0d", v, mem[0], mem[3], cyc);
    $finish;
  end
endmodule
"#,
        &[("XEZIM_EDGE_MERGE", "0")],
    );
    assert!(text.contains("GEN ff00ff00 7f 4f 20"), "answer:\n{text}");
    // Nine edge blocks (one counter, eight arms), each fired 20 times.
    assert!(
        stat(&text, "two_state_evals=") >= 9 * 20,
        "generate-arm stores left two-state:\n{text}"
    );
}

/// A block that reads an element nobody ever writes bails on x every
/// evaluation. The backoff must stop paying the executor entry for it:
/// after eight consecutive bails the attempt is skipped for a doubling
/// number of evaluations, so 2000 fires cost a few dozen attempts, not 2000.
#[test]
fn persistent_x_reader_backs_off() {
    let text = run(
        "xbail",
        r#"
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [7:0] mem [0:3];
  logic [7:0] acc = 0; logic [1:0] p = 0;
  always @(posedge clk) begin
    p <= p + 1;
    acc <= acc ^ mem[p];
  end
  initial begin
    repeat (2000) @(posedge clk);
    #1 $display("XB %b %0d", acc, p);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("XB xxxxxxxx 0"), "answer:\n{text}");
    let bails = stat(&text, "x_read=");
    assert!(
        bails <= 64,
        "the x-reading block was re-tried {bails} times:\n{text}"
    );
}
